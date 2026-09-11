mod errors;
mod job_registry;
mod transcoder;

use anyhow::{Context, Result};
use futures_util::{FutureExt, StreamExt};
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use std::sync::Arc;
use tokio::sync::Semaphore;
use redis::AsyncCommands;

// S3 and Cloudflare R2
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::Client as S3Client;

// Redis client
use redis_conn::RedisClient;

use job_registry::SharedRegistry;

use db::models::TranscodeJob;

#[cfg(target_os = "linux")]
fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting Linux io_uring transcoder service...");
    tokio_uring::start(async {
        if let Err(e) = run_transcoder().await {
            log::error!("Transcoder runtime error: {:?}", e);
        }
    });
}

#[cfg(not(target_os = "linux"))]
#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting fallback transcoder service...");
    if let Err(e) = run_transcoder().await {
        log::error!("Transcoder runtime error: {:?}", e);
    }
}

async fn run_transcoder() -> Result<()> {
    // Setup Redis connection
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = Arc::new(RedisClient::new_with_pool(&redis_url, 30).await.context("Failed to connect to Redis")?);

    // Setup RabbitMQ connection
    let rabbit_url = std::env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".to_string());
    let rabbit_client = rabbitmq_conn::RabbitMQClient::new_and_wait(&rabbit_url).await;
    let channel = rabbit_client.create_channel().await.context("Failed to create RabbitMQ channel")?;
    let _ = rabbitmq_conn::RabbitMQClient::declare_standard_queues(&channel).await?;

    // Setup S3 Config from s3-conn package
    let s3_config = s3_conn::S3Config::new().await.expect("Failed to initialize S3 config");
    let s3_client = Arc::new(s3_config.client);
    let r2_bucket = s3_config.bucket;

    let reqwest_client = reqwest::Client::new();
    let semaphore = Arc::new(Semaphore::new(3));
    let registry = SharedRegistry::new();

    let transcode_queue = rabbitmq_conn::get_transcode_queue();

    let mut consumer = channel
        .basic_consume(
            &transcode_queue,
            "transcoder-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .context("Failed to start RabbitMQ consume")?;

    log::info!("Consuming transcode jobs with concurrency limit 3...");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let job: TranscodeJob = match serde_json::from_slice(&delivery.data) {
                Ok(j) => j,
                Err(e) => {
                    log::error!("Failed to parse RabbitMQ message: {:?}", e);
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                    continue;
                }
            };
            
            println!("==> TRANSCODER: Received transcode job for video_id: {} resolution: {} chunk: {} (Waiting for semaphore)", job.video_id, job.resolution, job.chunk_index);

            let sem_clone = semaphore.clone();
            let channel_clone = channel.clone();
            let s3_client_clone = Arc::clone(&s3_client);
            let bucket_clone = r2_bucket.clone();
            let req_client_clone = reqwest_client.clone();
            let redis_client_clone = Arc::clone(&redis_client);
            let registry_clone = registry.clone();

            let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();
            let task_key = format!("chunk_{}_{}", job.chunk_index, job.resolution);
            
            // Register oneshot cancel channel in registry
            registry.register(job.video_id, task_key.clone(), abort_tx).await;

            let original_val = serde_json::to_value(&job).unwrap_or(serde_json::Value::Null);

            tokio::spawn(async move {
                // Acquire permit to limit concurrency
                let _permit = sem_clone.acquire().await.unwrap();
                log::info!("Acquired concurrency permit. Transcoding video_id: {} resolution: {} chunk: {}", job.video_id, job.resolution, job.chunk_index);
                println!("===============================================");
                println!("==> TRANSCODER: Starting transcode for video_id: {} resolution: {} chunk: {}", job.video_id, job.resolution, job.chunk_index);
                println!("===============================================");

                // Safe panic execution wrapper using AssertUnwindSafe + catch_unwind
                let transcode_future = std::panic::AssertUnwindSafe(async {
                    transcoder::execute_transcode(
                        job.video_id,
                        job.user_id,
                        job.chunk_index,
                        job.chunk_url.clone(),
                        job.resolution.clone(),
                        job.current_chunk,
                        job.total_chunks,
                        &req_client_clone,
                        &s3_client_clone,
                        &bucket_clone,
                        &channel_clone,
                        &redis_client_clone,
                        &registry_clone,
                        abort_rx,
                    ).await
                });

                let transcode_res = transcode_future.catch_unwind().await;

                // Unregister oneshot cancel channel
                registry_clone.unregister(job.video_id, &task_key).await;

                match transcode_res {
                    Ok(Ok(())) => {
                        log::info!("Transcode task completed successfully: video {} chunk {}", job.video_id, task_key);
                        println!("==> TRANSCODER: Finished transcode for video_id: {} resolution: {} chunk: {}", job.video_id, job.resolution, job.chunk_index);
                    }
                    Ok(Err(err)) => {
                        match err {
                            errors::TranscoderError::JobCancelled => {
                                log::warn!("Job video_id: {} resolution: {} chunk: {} cancelled by user", job.video_id, job.resolution, job.chunk_index);
                            }
                            _ => {
                                log::error!("Transcoding failed for video_id: {}, resolution: {}, chunk: {}: {:?}", job.video_id, job.resolution, job.chunk_index, err);
                                
                                let pool = redis_client_clone.get_pool().unwrap();
                                let mut redis_conn_guard = pool.get().await.unwrap();
                                let retry_key = format!("tus:transcoder:retries:{}:{}:{}", job.video_id, job.resolution, job.chunk_index);
                                let retries: u32 = redis_conn_guard.incr(&retry_key, 1).await.unwrap_or(0);
                                
                                if retries <= 3 {
                                    log::warn!("Requeuing transcode job (retry {})", retries);
                                    let _ = delivery.reject(lapin::options::BasicRejectOptions { requeue: true }).await;
                                    return;
                                } else {
                                    publish_to_dlq(
                                        &channel_clone,
                                        "transcoder-worker",
                                        &err.to_string(),
                                        &format!("video_{}_chunk_{}_{}", job.video_id, job.chunk_index, job.resolution),
                                        original_val,
                                    ).await;
                                }
                            }
                        }
                    }
                    Err(panic_payload) => {
                        let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                            *s
                        } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                            s.as_str()
                        } else {
                            "Unknown panic during transcode task execution"
                        };

                        log::error!("CRITICAL: Transcoding task panicked: {}", panic_msg);

                        publish_to_dlq(
                            &channel_clone,
                            "transcoder-worker",
                            &format!("PANIC: {}", panic_msg),
                            &format!("video_{}_chunk_{}_{}", job.video_id, job.chunk_index, job.resolution),
                            original_val,
                        ).await;
                    }
                }

                // Acknowledge original job from queue
                let _ = delivery.ack(BasicAckOptions::default()).await;
                
                let pool = redis_client_clone.get_pool().unwrap();
                let mut redis_conn_guard = pool.get().await.unwrap();
                let retry_key = format!("tus:transcoder:retries:{}:{}:{}", job.video_id, job.resolution, job.chunk_index);
                let _: () = redis_conn_guard.del(&retry_key).await.unwrap_or(());
                
                log::info!("Acknowledged transcode-jobs message for video: {} chunk: {}", job.video_id, job.chunk_index);
            });
        }
    }

    Ok(())
}

async fn publish_to_dlq(
    channel: &lapin::Channel,
    worker_name: &str,
    error_message: &str,
    task_id: &str,
    original_payload: serde_json::Value,
) {
    let dlq_payload = serde_json::json!({
        "worker_name": worker_name,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "error_message": error_message,
        "task_id": task_id,
        "original_payload": original_payload,
    });

    let _ = channel.basic_publish(
        "",
        "dead-letter-queue",
        BasicPublishOptions::default(),
        dlq_payload.to_string().as_bytes(),
        BasicProperties::default().with_delivery_mode(2),
    ).await;
    log::warn!("Published failed transcode task to Dead Letter Queue");
}
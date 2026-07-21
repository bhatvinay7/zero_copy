use anyhow::{Context, Result};
use futures_util::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use redis::AsyncCommands;
use std::sync::Arc;
#[cfg(target_os = "linux")]
use tokio_uring::fs::File;

// S3 and Cloudflare R2
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client as S3Client;
use db::models::{ChunkJob, TranscodeJob, DbUpdateEvent};

#[cfg(target_os = "linux")]
fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting io_uring worker...");

    tokio_uring::start(async {
        if let Err(e) = run_worker().await {
            log::error!("Worker error: {:?}", e);
        }
    });
}

#[cfg(not(target_os = "linux"))]
#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting standard worker...");

    if let Err(e) = run_worker().await {
        log::error!("Worker error: {:?}", e);
    }
}

async fn run_worker() -> Result<()> {
    // Setup Redis connection
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    println!("==> WORKER: Connecting to Redis URL: {}", redis_url);
    let redis_client = redis_conn::RedisClient::new_with_pool(&redis_url, 30).await.context("Failed to connect to Redis")?;
    let redis_pool = redis_client.get_pool().expect("Expected bb8 pool");

    // Setup RabbitMQ connection
    let rabbit_url = std::env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".to_string());
    
    let mut backoff = std::time::Duration::from_secs(1);
    let rabbit_conn = loop {
        match Connection::connect(&rabbit_url, ConnectionProperties::default()).await {
            Ok(conn) => {
                log::info!("Successfully connected to RabbitMQ");
                break conn;
            }
            Err(e) => {
                log::error!("Failed to connect to RabbitMQ: {}. Retrying in {}s...", e, backoff.as_secs());
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff * 2, std::time::Duration::from_secs(30));
            }
        }
    };
    
    let channel = rabbit_conn.create_channel().await.context("Failed to create RabbitMQ channel")?;
    let _ = rabbitmq_conn::RabbitMQClient::declare_standard_queues(&channel).await?;

    // Setup S3 Config from s3-conn package
    let s3_config = s3_conn::S3Config::new().await.expect("Failed to initialize S3 config");
    let s3_client = Arc::new(s3_config.client);
    let r2_bucket = s3_config.bucket;

    let mut consumer = channel
        .basic_consume(
            "chunk-queue",
            "io-uring-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .context("Failed to start RabbitMQ consume")?;

    let reqwest_client = reqwest::Client::new();

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let job: ChunkJob = match serde_json::from_slice(&delivery.data) {
                Ok(j) => j,
                Err(e) => {
                    log::error!("Failed to parse RabbitMQ message: {:?}", e);
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                    continue;
                }
            };

            log::info!("Received chunk job for video_id: {}", job.video_id);
            println!("===============================================");
            println!("==> WORKER: Received chunk job for video_id: {}", job.video_id);
            println!("===============================================");

            let original_val = serde_json::to_value(&job).unwrap_or(serde_json::Value::Null);

            // Publish ProcessingStart event
            let processing_start_event = DbUpdateEvent::ProcessingStart {
                video_id: job.video_id,
                user_id: job.user_id,
            };
            let processing_start_payload = serde_json::to_string(&processing_start_event).unwrap_or_default();
            let _ = channel.basic_publish(
                "",
                "db-updates",
                lapin::options::BasicPublishOptions::default(),
                processing_start_payload.as_bytes(),
                lapin::BasicProperties::default().with_delivery_mode(2),
            ).await;

            // Fetch and process chunks
            if let Err(e) = process_chunks(
                job.clone(),
                &reqwest_client,
                &s3_client,
                &r2_bucket,
                &redis_pool,
                &channel,
            ).await {
                log::error!("Error processing video chunks: {:?}", e);
                
                let retry_key = format!("tus:worker:retries:{}", job.video_id);
                let mut redis_conn = redis_pool.get().await.unwrap();
                let retries: u32 = redis_conn.incr(&retry_key, 1).await.unwrap_or(0);
                
                if retries <= 3 {
                    log::warn!("Requeuing chunk job for video_id: {} (retry {})", job.video_id, retries);
                    let _ = delivery.reject(lapin::options::BasicRejectOptions { requeue: true }).await;
                    continue;
                } else {
                    publish_to_dlq(
                        &channel,
                        "io-uring-worker",
                        &e.to_string(),
                        &job.video_id.to_string(),
                        original_val,
                    ).await;
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
            } else {
                // Acknowledge original chunk-queue job
                let _ = delivery.ack(BasicAckOptions::default()).await;
                let retry_key = format!("tus:worker:retries:{}", job.video_id);
                let mut redis_conn = redis_pool.get().await.unwrap();
                let _: () = redis_conn.del(&retry_key).await.unwrap_or(());
                log::info!("Processed and completed chunk job");
                println!("==> WORKER: Processed and completed chunk job for video_id: {}", job.video_id);
            }
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
    log::warn!("Published failed task to Dead Letter Queue");
}

async fn process_chunks(
    job: ChunkJob,
    req_client: &reqwest::Client,
    s3_client: &Arc<S3Client>,
    bucket: &str,
    redis_pool: &redis_conn::bb8::Pool<redis_conn::bb8_redis::RedisConnectionManager>,
    channel: &lapin::Channel,
) -> Result<()> {
    log::info!("Downloading full video {} for segmenting...", job.video_id);
    let temp_orig_filename = format!("temp_orig_{}.mp4", job.video_id);

    // Download entire video
    let response = req_client.get(&job.download_url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to download video: status {}", response.status()));
    }
    let bytes = response.bytes().await?;
    
    // Use the existing write_chunk_to_disk helper which handles io_uring on linux
    write_chunk_to_disk(&temp_orig_filename, bytes).await?;

    log::info!("Video downloaded. Segmenting with ffmpeg...");

    // Output pattern for segments
    let chunk_pattern = format!("chunk_{}_%03d.mp4", job.video_id);

    // Run ffmpeg to segment the video
    let status = tokio::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i", &temp_orig_filename,
            "-f", "segment",
            "-segment_time", "10",
            "-c", "copy",
            "-reset_timestamps", "1",
            &chunk_pattern,
        ])
        .status()
        .await
        .context("Failed to execute ffmpeg for segmenting")?;

    if !status.success() {
        let _ = tokio::fs::remove_file(&temp_orig_filename).await;
        return Err(anyhow::anyhow!("FFmpeg segmenting failed with status: {}", status));
    }

    // Read generated chunks
    let mut chunks = Vec::new();
    let mut dir = tokio::fs::read_dir(".").await?;
    let prefix = format!("chunk_{}_", job.video_id);
    while let Some(entry) = dir.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) && name.ends_with(".mp4") {
            chunks.push(name);
        }
    }

    // Sort to ensure correct order
    chunks.sort();
    let total_chunks = chunks.len() as u32;

    if total_chunks == 0 {
        let _ = tokio::fs::remove_file(&temp_orig_filename).await;
        return Err(anyhow::anyhow!("FFmpeg produced 0 chunks"));
    }

    log::info!("Segmenting complete. Produced {} chunks. Uploading...", total_chunks);

    for (i, chunk_filename) in chunks.into_iter().enumerate() {
        let chunk_index = (i + 1) as u32;
        
        let chunk_id = format!("video_{}_chunk_{}", job.video_id, chunk_index);
        let processing_flag_key = format!("tus:worker:chunk:{}:processing", chunk_id);

        let mut redis_conn = redis_pool.get().await.unwrap();
        let is_processed_res = redis_conn.get::<_, Option<String>>(&processing_flag_key).await;
        let is_processed = is_processed_res.unwrap_or(None);

        if is_processed == Some("complete".to_string()) {
            log::info!("Chunk {} already fully processed. Skipping.", chunk_index);
            let _ = tokio::fs::remove_file(&chunk_filename).await;
            continue;
        }

        let _: () = redis_conn.set_ex(&processing_flag_key, "processing", 3600).await.unwrap_or(());

        let chunk_key = format!("chunks/{}/chunk_{}.mp4", job.video_id, chunk_index);

        log::info!("Uploading {} to R2...", chunk_filename);
        let body_stream = aws_sdk_s3::primitives::ByteStream::from_path(&chunk_filename).await?;
        
        s3_client
            .put_object()
            .bucket(bucket.to_string())
            .key(&chunk_key)
            .body(body_stream)
            .content_type("video/mp4")
            .send()
            .await?;
        
        log::info!("Successfully uploaded chunk {} to R2", chunk_key);
        let _ = tokio::fs::remove_file(&chunk_filename).await;

        let presign_config = PresigningConfig::expires_in(std::time::Duration::from_secs(86400))?;
        let presigned = s3_client
            .get_object()
            .bucket(bucket)
            .key(&chunk_key)
            .presigned(presign_config)
            .await?;
        let chunk_url = presigned.uri().to_string();

        let active_key = format!("tus:video:{}:active", job.video_id);
        let mut redis_conn = redis_pool.get().await.unwrap();
        let is_active = redis_conn.get::<_, Option<bool>>(&active_key).await.unwrap_or(Some(false)).unwrap_or(false);
        if !is_active {
            log::warn!("Video {} was cancelled, dropping chunk publishing", job.video_id);
            break;
        }

        let resolutions = vec![
            "UHD_4K".to_string(),
            "FHD_1080P".to_string(),
            "HD_720P".to_string(),
            "SD_480P".to_string(),
            "SD_360P".to_string(),
        ];

        for resolution in resolutions {
            let transcode_payload = TranscodeJob {
                video_id: job.video_id,
                user_id: job.user_id,
                chunk_index,
                chunk_url: chunk_url.clone(),
                resolution: resolution.clone(),
                current_chunk: chunk_index,
                total_chunks,
            };

            let transcode_body = serde_json::to_string(&transcode_payload)?;
            channel
                .basic_publish(
                    "",
                    "transcode-jobs",
                    BasicPublishOptions::default(),
                    transcode_body.as_bytes(),
                    BasicProperties::default().with_delivery_mode(2),
                )
                .await?;
            log::info!("Published transcode job for resolution: {}", resolution);
        }

        let mut redis_conn = redis_pool.get().await.unwrap();
        let _: () = redis_conn.set(&processing_flag_key, "complete").await.unwrap_or(());
    }

    let _ = tokio::fs::remove_file(&temp_orig_filename).await;

    Ok(())
}

#[cfg(target_os = "linux")]
async fn write_chunk_to_disk(filename: &str, bytes: bytes::Bytes) -> Result<()> {
    let file = File::create(filename)
        .await
        .context("Failed to create local chunk file via io_uring")?;
    let (res, _) = file.write_all_at(bytes.to_vec(), 0).await;
    res.context("io_uring failed writing chunk to disk")?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn write_chunk_to_disk(filename: &str, bytes: bytes::Bytes) -> Result<()> {
    tokio::fs::write(filename, bytes)
        .await
        .context("Failed to write chunk to disk")?;
    Ok(())
}

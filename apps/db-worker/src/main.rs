use anyhow::{Context, Result};
use futures_util::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use redis::AsyncCommands;
use db::DbPool;
use std::sync::Arc;

use db::models::{DbUpdateEvent, DbUpdateMessage, MergeJob, NewVideoFormat};

#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting db-worker...");

    if let Err(e) = run_db_worker().await {
        log::error!("DB-worker runtime error: {:?}", e);
    }
}

async fn run_db_worker() -> Result<()> {
    // Setup PostgreSQL DB Pool
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = Arc::new(db::establish_connection_pool(&db_url).expect("Failed to connect to PostgreSQL Database"));

    // Setup Redis connection
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = Arc::new(redis_conn::RedisClient::new_with_pool(&redis_url, 30).await.context("Failed to connect to Redis")?);
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

    let mut consumer = channel
        .basic_consume(
            "db-updates",
            "db-update-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .context("Failed to start RabbitMQ db-updates consume")?;

    log::info!("Listening for db-update jobs...");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let event: DbUpdateEvent = match serde_json::from_slice(&delivery.data) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to parse RabbitMQ message: {:?}", e);
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                    continue;
                }
            };

            let original_val = serde_json::to_value(&event).unwrap_or(serde_json::Value::Null);
            let (task_id, result) = match event.clone() {
                DbUpdateEvent::ProcessingStart { video_id, user_id } => {
                    log::info!("Processing ProcessingStart for video_id: {}", video_id);
                    (
                        format!("video_{}_processing_start", video_id),
                        handle_processing_start(video_id, user_id, &redis_pool).await
                    )
                },
                DbUpdateEvent::ChunkComplete(msg) => {
                    log::info!("Processing ChunkComplete for video_id: {} resolution: {} chunk: {}", msg.video_id, msg.resolution, msg.chunk_index);
                    (
                        format!("video_{}_chunk_{}_{}", msg.video_id, msg.chunk_index, msg.resolution),
                        handle_chunk_complete(msg, &redis_client, &redis_pool, &channel).await
                    )
                },
                DbUpdateEvent::MergeComplete { video_id, user_id, resolution, url } => {
                    log::info!("Processing MergeComplete for video_id: {} resolution: {}", video_id, resolution);
                    (
                        format!("video_{}_merge_{}", video_id, resolution),
                        handle_merge_complete(video_id, user_id, resolution, url, &db_pool, &redis_client, &redis_pool).await
                    )
                }
            };

            if let Err(e) = result {
                log::error!("Error writing db update: {:?}", e);
                
                let retry_key = format!("tus:dbworker:retries:{}", task_id);
                let mut redis_conn = redis_pool.get().await.unwrap();
                let retries: u32 = redis_conn.incr(&retry_key, 1).await.unwrap_or(0);
                
                if retries <= 3 {
                    log::warn!("Requeuing db update job {} (retry {})", task_id, retries);
                    let _ = delivery.reject(lapin::options::BasicRejectOptions { requeue: true }).await;
                    continue;
                } else {
                    publish_to_dlq(&channel, "db-worker", &e.to_string(), &task_id, original_val).await;
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
            } else {
                let _ = delivery.ack(BasicAckOptions::default()).await;
                let retry_key = format!("tus:dbworker:retries:{}", task_id);
                let mut redis_conn = redis_pool.get().await.unwrap();
                let _: () = redis_conn.del(&retry_key).await.unwrap_or(());
                log::info!("Acknowledged db-update message: {}", task_id);
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
    log::warn!("Published failed DB update task {} to DLQ", task_id);
}

async fn handle_chunk_complete(
    msg: DbUpdateMessage,
    redis_client: &redis_conn::RedisClient,
    redis_pool: &redis_conn::bb8::Pool<redis_conn::bb8_redis::RedisConnectionManager>,
    channel: &lapin::Channel,
) -> Result<()> {
    let mut redis_conn = redis_pool.get().await?;
    
    // 1. Log completed chunk in Redis set
    let chunks_set_key = format!("tus:video:{}:res:{}:chunks", msg.video_id, msg.resolution);
    let _: () = redis_conn.sadd(&chunks_set_key, msg.chunk_index).await?;
    let completed_chunks: u32 = redis_conn.scard(&chunks_set_key).await?;

    log::info!("Video: {} resolution: {} progress: {}/{}", msg.video_id, msg.resolution, completed_chunks, msg.total_chunks);

    // Update SSE progress!
    let _ = redis_client.update_video_progress(msg.video_id, msg.user_id, &msg.resolution, completed_chunks, msg.total_chunks).await;

    // 2. If all chunks are completed for this resolution
    if completed_chunks == msg.total_chunks {
        log::info!("All {} chunks completed for video {} resolution {}. Publishing MergeJob...", msg.total_chunks, msg.video_id, msg.resolution);

        let merge_job = MergeJob {
            video_id: msg.video_id,
            user_id: msg.user_id,
            resolution: msg.resolution.clone(),
            total_chunks: msg.total_chunks,
        };

        let merge_payload = serde_json::to_string(&merge_job)?;
        channel.basic_publish(
            "",
            "merge-jobs",
            BasicPublishOptions::default(),
            merge_payload.as_bytes(),
            BasicProperties::default().with_delivery_mode(2),
        ).await?;
        
        log::info!("Published MergeJob for video {} resolution {}", msg.video_id, msg.resolution);
    }

    Ok(())
}

async fn handle_merge_complete(
    video_id: i32,
    user_id: i32,
    resolution: String,
    url: String,
    db_pool: &DbPool,
    redis_client: &redis_conn::RedisClient,
    redis_pool: &redis_conn::bb8::Pool<redis_conn::bb8_redis::RedisConnectionManager>,
) -> Result<()> {
    let mut db_conn = db_pool.get()?;
    
    // 1. Write the new video format to DB
    let new_format = NewVideoFormat {
        video_id,
        resolution: resolution.clone(),
        url,
    };
    db::add_video_format(&mut db_conn, &new_format)?;
    log::info!("Saved NewVideoFormat for video {} resolution {}", video_id, resolution);

    // 2. Update completion state
    let mut redis_conn = redis_pool.get().await?;
    let resolutions_set_key = format!("tus:video:{}:completed_resolutions", video_id);
    let _: () = redis_conn.sadd(&resolutions_set_key, &resolution).await?;
    let completed_resolutions: u32 = redis_conn.scard(&resolutions_set_key).await?;

    if completed_resolutions == 5 {
        log::info!("All 5 resolutions completely merged for video {}. Finalizing status...", video_id);
        
        // Finalize status in DB
        db::update_video_status(&mut db_conn, video_id, "completed")?;

        // Cleanup Redis keys
        let _: () = redis_conn.del(&resolutions_set_key).await?;
        let resolutions = vec!["UHD_4K", "FHD_1080P", "HD_720P", "SD_480P", "SD_360P"];
        for res in resolutions {
            let res_key = format!("tus:video:{}:res:{}:chunks", video_id, res);
            let _: () = redis_conn.del(&res_key).await?;
        }
        
        let _ = redis_client.clean_video_progress(video_id).await;
        
        // Force a 100% SSE event so the frontend knows to finish!
        let payload = serde_json::json!({
            "user_id": user_id,
            "video_id": video_id,
            "percentage": 100,
            "resolutions": {},
        });
        let _: () = redis_conn.publish("video:transcode:progress", payload.to_string()).await?;
        log::info!("Published final 100% completion SSE for video {}", video_id);
    }
    
    Ok(())
}

async fn handle_processing_start(
    video_id: i32,
    user_id: i32,
    redis_pool: &redis_conn::bb8::Pool<redis_conn::bb8_redis::RedisConnectionManager>,
) -> Result<()> {
    // 2. Publish initial 1% progress to SSE so UI updates immediately from 'queued' to 'processing'
    let mut redis_conn = redis_pool.get().await?;
    let payload = serde_json::json!({
        "user_id": user_id,
        "video_id": video_id,
        "percentage": 1, 
        "resolutions": {},
    });
    let _: () = redis_conn.publish("video:transcode:progress", payload.to_string()).await?;
    log::info!("Published initial processing start SSE for video {}", video_id);

    Ok(())
}

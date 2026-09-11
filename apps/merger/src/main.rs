use anyhow::{Context, Result};
use db::models::MergeJob;
use futures_util::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use redis::AsyncCommands;
use std::sync::Arc;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client as S3Client;

#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting merger service...");
    if let Err(e) = run_merger().await {
        log::error!("Merger runtime error: {:?}", e);
    }
}

async fn run_merger() -> Result<()> {
    // Setup Redis connection
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
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

    // Consume merge-jobs
    let mut consumer = channel
        .basic_consume(
            "merge-jobs",
            "merger-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .context("Failed to start RabbitMQ merge-jobs consume")?;

    log::info!("Listening for merge jobs...");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let msg: MergeJob = match serde_json::from_slice(&delivery.data) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to parse MergeJob: {:?}", e);
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                    continue;
                }
            };
            
            log::info!("Processing merge-job for video: {} resolution: {}", msg.video_id, msg.resolution);

            if let Err(e) = handle_merge(msg.clone(), &s3_client, &r2_bucket, &channel).await {
                log::error!("Failed to merge chunks for video {} res {}: {:?}", msg.video_id, msg.resolution, e);
                
                let retry_key = format!("tus:merger:retries:{}:{}", msg.video_id, msg.resolution);
                let mut redis_conn = redis_pool.get().await.unwrap();
                let retries: u32 = redis_conn.incr(&retry_key, 1).await.unwrap_or(0);
                
                if retries <= 3 {
                    log::warn!("Requeuing merge job (retry {})", retries);
                    let _ = delivery.reject(lapin::options::BasicRejectOptions { requeue: true }).await;
                    continue;
                } else {
                    let dlq_payload = serde_json::json!({
                        "worker_name": "merger",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "error_message": e.to_string(),
                        "task_id": format!("video_{}_merge_{}", msg.video_id, msg.resolution),
                        "original_payload": msg,
                    });
                    
                    let _ = channel.basic_publish(
                        "",
                        "dead-letter-queue",
                        BasicPublishOptions::default(),
                        dlq_payload.to_string().as_bytes(),
                        BasicProperties::default().with_delivery_mode(2),
                    ).await;
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
            } else {
                let _ = delivery.ack(BasicAckOptions::default()).await;
                let retry_key = format!("tus:merger:retries:{}:{}", msg.video_id, msg.resolution);
                let mut redis_conn = redis_pool.get().await.unwrap();
                let _: () = redis_conn.del(&retry_key).await.unwrap_or(());
            }
        }
    }

    Ok(())
}

async fn handle_merge(
    job: MergeJob,
    s3_client: &Arc<S3Client>,
    bucket: &str,
    channel: &lapin::Channel,
) -> Result<()> {
    log::info!("Starting merge for video {} resolution {}", job.video_id, job.resolution);
    
    // 1. Download all chunks
    let mut list_content = String::new();
    let mut local_files = Vec::new();

    for i in 1..=job.total_chunks {
        let chunk_key = format!("transcoded/{}/{}/chunk_{}.mp4", job.video_id, job.resolution, i);
        let local_filename = format!("tmp_merge_{}_{}_chunk_{}.mp4", job.video_id, job.resolution, i);
        
        log::info!("Downloading {}...", chunk_key);
        let mut object = s3_client
            .get_object()
            .bucket(bucket)
            .key(&chunk_key)
            .send()
            .await?;
            
        let mut file = tokio::fs::File::create(&local_filename).await?;
        while let Some(bytes) = object.body.try_next().await? {
            tokio::io::AsyncWriteExt::write_all(&mut file, &bytes).await?;
        }
        
        // Escape backslashes for ffmpeg if needed, but relative path in current dir doesn't strictly need it.
        list_content.push_str(&format!("file '{}'\n", local_filename));
        local_files.push(local_filename);
    }
    
    // 2. Write list.txt
    let list_filename = format!("tmp_list_{}_{}.txt", job.video_id, job.resolution);
    tokio::fs::write(&list_filename, list_content).await?;
    
    // 3. Run FFmpeg concat
    let output_filename = format!("tmp_final_{}_{}.mp4", job.video_id, job.resolution);
    log::info!("Running ffmpeg concat for video {} resolution {}...", job.video_id, job.resolution);
    
    let status = tokio::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-f", "concat",
            "-safe", "0",
            "-i", &list_filename,
            "-c", "copy",
            &output_filename,
        ])
        .status()
        .await
        .context("Failed to execute ffmpeg concat")?;
        
    if !status.success() {
        // Cleanup local files
        let _ = tokio::fs::remove_file(&list_filename).await;
        for file in &local_files { let _ = tokio::fs::remove_file(file).await; }
        return Err(anyhow::anyhow!("FFmpeg concat failed with status: {}", status));
    }
    
    // 4. Upload final video to R2
    let final_key = format!("final/{}/{}.mp4", job.video_id, job.resolution);
    log::info!("Uploading final video to {}...", final_key);
    
    let body_stream = aws_sdk_s3::primitives::ByteStream::from_path(&output_filename).await?;
    s3_client
        .put_object()
        .bucket(bucket)
        .key(&final_key)
        .body(body_stream)
        .content_type("video/mp4")
        .send()
        .await?;
        
    // 5. Generate URL and publish MergeComplete to db-updates
    let presign_config = PresigningConfig::expires_in(std::time::Duration::from_secs(86400))?;
    let presigned = s3_client
        .get_object()
        .bucket(bucket)
        .key(&final_key)
        .presigned(presign_config)
        .await?;
    let final_url = presigned.uri().to_string();
    
    let merge_complete_msg = db::models::DbUpdateEvent::MergeComplete {
        video_id: job.video_id,
        user_id: job.user_id,
        resolution: job.resolution.clone(),
        url: final_url,
    };
    let completion_body = serde_json::to_string(&merge_complete_msg)?;
    
    channel
        .basic_publish(
            "",
            "db-updates",
            lapin::options::BasicPublishOptions::default(),
            completion_body.as_bytes(),
            lapin::BasicProperties::default().with_delivery_mode(2),
        )
        .await?;
    log::info!("Published MergeComplete for video {} resolution {}", job.video_id, job.resolution);
    
    // 6. Cleanup Local Files
    let _ = tokio::fs::remove_file(&list_filename).await;
    let _ = tokio::fs::remove_file(&output_filename).await;
    for file in &local_files { let _ = tokio::fs::remove_file(file).await; }
    
    // 7. Cleanup R2 Chunks
    for i in 1..=job.total_chunks {
        let chunk_key = format!("transcoded/{}/{}/chunk_{}.mp4", job.video_id, job.resolution, i);
        let _ = s3_client.delete_object().bucket(bucket).key(&chunk_key).send().await;
    }

    Ok(())
}

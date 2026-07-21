use crate::errors::TranscoderError;
use crate::job_registry::SharedRegistry;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client as S3Client;
use lapin::options::BasicPublishOptions;
use lapin::BasicProperties;
use redis_conn::RedisClient;
use tokio::sync::oneshot;

use db::models::DbUpdateMessage;

#[cfg(target_os = "linux")]
async fn download_and_write_zero_copy(
    url_str: &str,
    path: &str,
) -> Result<(), anyhow::Error> {
    use tokio_uring::fs::File;
    use futures_util::StreamExt;

    // Use reqwest to handle HTTPS TLS session negotiations natively
    let client = reqwest::Client::new();
    let mut stream = client.get(url_str).send().await?.bytes_stream();

    // Create file via io_uring
    let file = File::create(path).await?;
    let mut file_offset = 0u64;

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res?;
        
        // Write chunk directly to disk via io_uring
        let (write_res, _) = file.write_at(chunk.to_vec(), file_offset).await;
        write_res?;
        file_offset += chunk.len() as u64;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn download_and_write_zero_copy(
    url_str: &str,
    path: &str,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let response = client.get(url_str).send().await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(path, bytes).await?;
    Ok(())
}

pub async fn execute_transcode(
    video_id: i32,
    user_id: i32,
    chunk_index: u32,
    chunk_url: String,
    resolution: String,
    current_chunk: u32,
    total_chunks: u32,
    _req_client: &reqwest::Client,
    s3_client: &S3Client,
    bucket: &str,
    channel: &lapin::Channel,
    redis_client: &RedisClient,
    _registry: &SharedRegistry,
    mut abort_rx: oneshot::Receiver<()>,
) -> Result<(), TranscoderError> {
    let task_key = format!("chunk_{}_{}", chunk_index, resolution);

    // Double check that the video is active before downloading
    if !redis_client.is_video_active(video_id).await? {
        log::warn!("Video transcode {} was cancelled. Dropping chunk task {}.", video_id, task_key);
        return Ok(());
    }

    let input_filename = format!("transcode_in_{}_{}_{}.mp4", video_id, chunk_index, resolution);
    let output_filename = format!("transcode_out_{}_{}_{}.mp4", video_id, chunk_index, resolution);

    log::info!("Downloading source chunk zero-copy from {}", chunk_url);

    // Download chunk locally using io_uring zero-copy or standard fallback
    download_and_write_zero_copy(&chunk_url, &input_filename).await?;

    // Check cancellation status after download complete
    if abort_rx.try_recv().is_ok() {
        let _ = tokio::fs::remove_file(&input_filename).await;
        return Err(TranscoderError::JobCancelled);
    }

    // Determine scale settings
    let scale = match resolution.as_str() {
        "UHD_4K" => "3840:2160",
        "FHD_1080P" => "1920:1080",
        "HD_720P" => "1280:720",
        "SD_480P" => "854:480",
        "SD_360P" => "640:360",
        _ => "1280:720",
    };

    log::info!("Spawning ffmpeg to transcode {} to scale: {}", task_key, scale);

    // Spawn FFmpeg process
    let mut child = tokio::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i", &input_filename,
            "-vf", &format!("scale={}", scale),
            "-c:a", "copy",
            &output_filename,
        ])
        .spawn()
        .map_err(anyhow::Error::new)?;

    // Monitor child process wait vs cancellation oneshot trigger
    let status_res = tokio::select! {
        status = child.wait() => {
            status.map_err(anyhow::Error::new)?
        }
        _ = &mut abort_rx => {
            log::warn!("Cancellation signal received. Terminating ffmpeg child process for task: {}", task_key);
            let _ = child.kill().await;
            // Clean up temporary files on user cancellation
            let _ = tokio::fs::remove_file(&input_filename).await;
            let _ = tokio::fs::remove_file(&output_filename).await;
            return Err(TranscoderError::JobCancelled);
        }
    };

    // Clean up input chunk on successful transcode
    let _ = tokio::fs::remove_file(&input_filename).await;

    if !status_res.success() {
        // Do NOT delete output_filename on error to allow developers to inspect the corrupted output chunk
        return Err(anyhow::anyhow!("FFmpeg exited with non-zero status: {}", status_res).into());
    }

    log::info!("Transcode successful for {}. Uploading output chunk to R2...", task_key);

    // Upload output chunk to Cloudflare R2
    let destination_key = format!("transcoded/{}/{}/chunk_{}.mp4", video_id, resolution, chunk_index);
    
    // We wrap reading file and upload. Check if upload is cancelled during R2 uploads.
    if abort_rx.try_recv().is_ok() {
        let _ = tokio::fs::remove_file(&output_filename).await;
        return Err(TranscoderError::JobCancelled);
    }

    let body_stream = aws_sdk_s3::primitives::ByteStream::from_path(&output_filename)
        .await
        .map_err(anyhow::Error::new)?;

    s3_client
        .put_object()
        .bucket(bucket)
        .key(&destination_key)
        .body(body_stream)
        .content_type("video/mp4")
        .send()
        .await
        .map_err(anyhow::Error::new)?;

    // Clean up output file on successful R2 upload completion
    let _ = tokio::fs::remove_file(&output_filename).await;

    // Check again
    if abort_rx.try_recv().is_ok() {
        return Err(TranscoderError::JobCancelled);
    }

    // Generate presigned GET URL for this transcoded chunk
    let presign_config = PresigningConfig::expires_in(std::time::Duration::from_secs(86400)).map_err(anyhow::Error::new)?;
    let presigned = s3_client
        .get_object()
        .bucket(bucket)
        .key(&destination_key)
        .presigned(presign_config)
        .await
        .map_err(anyhow::Error::new)?;
    let transcoded_url = presigned.uri().to_string();

    // Publish completed message to db-updates
    let chunk_msg = DbUpdateMessage {
        video_id,
        user_id,
        chunk_index,
        resolution: resolution.clone(),
        transcoded_url,
        current_chunk,
        total_chunks,
    };
    let completion_payload = db::models::DbUpdateEvent::ChunkComplete(chunk_msg);

    let completion_body = serde_json::to_string(&completion_payload).map_err(anyhow::Error::new)?;
    channel
        .basic_publish(
            "",
            "db-updates",
            BasicPublishOptions::default(),
            completion_body.as_bytes(),
            BasicProperties::default().with_delivery_mode(2),
        )
        .await
        .map_err(anyhow::Error::new)?;

    log::info!("Published completion message to db-updates for video {} resolution {}", video_id, resolution);

    Ok(())
}

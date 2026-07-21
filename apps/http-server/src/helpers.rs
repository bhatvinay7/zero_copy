use anyhow::{Context, Result};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client as S3Client;
use axum::body::Bytes;

pub async fn upload_to_r2(client: &S3Client, bucket: &str, key: &str, body: Bytes, content_type: &str) -> Result<()> {
    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(aws_sdk_s3::primitives::ByteStream::from(body))
        .content_type(content_type)
        .send()
        .await
        .context("Failed to upload to Cloudflare R2")?;
    Ok(())
}

pub async fn upload_to_s3(client: &S3Client, bucket: &str, key: &str, body: Bytes, content_type: &str) -> Result<()> {
    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(aws_sdk_s3::primitives::ByteStream::from(body))
        .content_type(content_type)
        .send()
        .await
        .context("Failed to upload to AWS S3")?;
    Ok(())
}

pub async fn get_presigned_url_r2(client: &S3Client, bucket: &str, key: &str, expires_in: std::time::Duration) -> Result<String> {
    let presign_config = PresigningConfig::expires_in(expires_in)
        .context("Failed to create presigning configuration")?;
    let presigned = client.get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presign_config)
        .await
        .context("Failed to generate presigned GET URL for R2")?;
    Ok(presigned.uri().to_string())
}

#[allow(non_snake_case)]
pub fn DateNowMs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

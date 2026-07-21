use anyhow::{Context, Result};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client as S3Client;

pub async fn get_presigned_url_r2(
    client: &S3Client,
    bucket: &str,
    key: &str,
    expires_in: std::time::Duration,
) -> Result<String> {
    let presign_config = PresigningConfig::expires_in(expires_in)
        .context("Failed to create presigning config")?;
    let presigned = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presign_config)
        .await
        .context("Failed to generate presigned URL")?;
    Ok(presigned.uri().to_string())
}

#[allow(non_snake_case)]
pub fn DateNowMs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

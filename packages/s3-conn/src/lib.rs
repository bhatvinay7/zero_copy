pub use aws_sdk_s3;
pub use aws_config;

use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::Client as S3Client;

#[derive(Clone, Debug)]
pub struct S3Config {
    pub client: S3Client,
    pub bucket: String,
    pub public_url: String,
}

impl S3Config {
    /// Loads the S3 / Cloudflare R2 configuration from the environment and creates a client.
    pub async fn new() -> Result<Self> {
        let access_key = std::env::var("CLOUDFLARE_R2_ACCESS_KEY_ID")
            .unwrap_or_default()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim()
            .to_string();
        let secret_key = std::env::var("CLOUDFLARE_R2_SECRET_ACCESS_KEY")
            .unwrap_or_default()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim()
            .to_string();
        let bucket = std::env::var("CLOUDFLARE_R2_BUCKET")
            .unwrap_or_else(|_| "transcoder-bucket".to_string())
            .trim_matches(|c| c == '"' || c == '\'')
            .trim()
            .to_string();
        
        // Some services use public URL, some use endpoint url for Cloudflare R2
        let endpoint_url = std::env::var("CLOUDFLARE_R2_PUBLIC_URL")
            .unwrap_or_default()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim()
            .to_string();

        let credentials = Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "StaticCredentials",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("auto"))
            .credentials_provider(credentials)
            .endpoint_url(&endpoint_url)
            .load()
            .await;

        let client = S3Client::new(&config);

        Ok(Self {
            client,
            bucket,
            public_url: endpoint_url,
        })
    }

    /// Deletes a specific object by key
    pub async fn delete_object(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    /// Deletes all objects under a given prefix
    pub async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let mut object_stream = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .into_paginator()
            .send();

        while let Some(res) = object_stream.next().await {
            let page = res?;
            for obj in page.contents() {
                if let Some(key) = obj.key() {
                    let _ = self.delete_object(key).await;
                }
            }
        }
        Ok(())
    }
}

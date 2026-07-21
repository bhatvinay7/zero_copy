use anyhow::{Context, Result};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
pub use bb8;
pub use bb8_redis;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
    manager: ConnectionManager,
    pool: Option<Pool<RedisConnectionManager>>,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .context("Failed to open connection to Redis")?;
            
        let manager = ConnectionManager::new(client.clone())
            .await
            .context("Failed to create Redis connection manager")?;
            
        // Spawn a keep-alive task to prevent idle connections from being dropped by managed Redis services.
        let mut keep_alive_conn = manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let res: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut keep_alive_conn).await;
                if let Err(e) = res {
                    eprintln!("Redis keep-alive ping failed: {}", e);
                }
            }
        });
            
        Ok(Self { client, manager, pool: None })
    }

    pub async fn new_with_pool(redis_url: &str, max_connections: u32) -> Result<Self> {
        let mut client = Self::new(redis_url).await?;
        
        let pool_manager = RedisConnectionManager::new(redis_url)
            .context("Failed to create bb8 redis manager")?;
        
        let pool = bb8::Pool::builder()
            .max_size(max_connections)
            .build(pool_manager)
            .await
            .context("Failed to build bb8 redis pool")?;
            
        client.pool = Some(pool);
        Ok(client)
    }

    pub fn get_client(&self) -> &redis::Client {
        &self.client
    }

    pub fn get_connection_manager(&self) -> ConnectionManager {
        self.manager.clone()
    }

    pub fn get_pool(&self) -> Option<Pool<RedisConnectionManager>> {
        self.pool.clone()
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.manager.clone();
        let val: Option<String> = conn.get(key).await?;
        Ok(val)
    }

    pub async fn set(&self, key: &str, value: &str, expiry_seconds: Option<u64>) -> Result<()> {
        let mut conn = self.manager.clone();
        if let Some(expiry) = expiry_seconds {
            let _: () = conn.set_ex(key, value, expiry).await?;
        } else {
            let _: () = conn.set(key, value).await?;
        }
        Ok(())
    }

    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        let _: () = conn.publish(channel, message).await?;
        Ok(())
    }

    /// Delete one or more keys.
    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        let _: () = conn.del(key).await?;
        Ok(())
    }

    pub async fn get_hash_all(&self, key: &str) -> Result<std::collections::HashMap<String, u32>> {
        let mut conn = self.manager.clone();
        let hash: std::collections::HashMap<String, u32> = conn.hgetall(key).await?;
        Ok(hash)
    }

    /// Atomically increments the video's completed chunks count, verifies if the progress
    /// has increased by at least 5% (or reached 100%), updates the dashboard stats,
    /// and publishes a JSON event payload to the redis pub/sub channel.
    pub async fn update_video_progress(
        &self,
        video_id: i32,
        user_id: i32,
        resolution: &str,
        completed_chunks: u32,
        total_chunks: u32,
    ) -> Result<Option<u32>> {
        let mut conn = self.manager.clone();

        // Calculate progress percentage for this specific resolution
        let percent = if total_chunks == 0 { 0 } else { (completed_chunks * 100) / total_chunks };

        // Save into a hash: tus:video:{video_id}:resolutions
        let hash_key = format!("tus:video:{}:resolutions", video_id);
        let _: () = conn.hset(&hash_key, resolution, percent).await?;

        // Also track overall last published average, or just publish the full hash
        // For backwards compatibility and dashboard tracking:
        let all_res: std::collections::HashMap<String, u32> = conn.hgetall(&hash_key).await?;
        
        let mut sum = 0;
        let mut count = 0;
        for (_, p) in all_res.iter() {
            sum += p;
            count += 1;
        }
        
        // If we expect 5 resolutions (4K, 1080p, 720p, 480p, 360p), we divide by 5 for a total percentage.
        // Or we just divide by `count` if we want average of started tasks, but 5 is absolute.
        let total_percent = sum / 5;

        // Publish the structured JSON event with the per-resolution map!
        let payload = serde_json::json!({
            "user_id": user_id,
            "video_id": video_id,
            "percentage": total_percent,
            "resolutions": all_res,
        });

        let _: () = conn.publish("video:transcode:progress", payload.to_string()).await?;
        
        // Also update the global ongoing hash
        let _: () = conn.hset("tus:video:ongoing", video_id.to_string(), total_percent.to_string()).await?;
        let last_percent_key = format!("tus:video:{}:last_published_percent", video_id);
        let _: () = conn.set(&last_percent_key, total_percent).await?;

        Ok(Some(total_percent))
    }

    /// Clean up transcode tracking state in Redis when a video transcode completes.
    pub async fn clean_video_progress(&self, video_id: i32) -> Result<()> {
        let mut conn = self.manager.clone();
        let last_percent_key = format!("tus:video:{}:last_published_percent", video_id);
        let hash_key = format!("tus:video:{}:resolutions", video_id);
        let _: () = conn.del(&last_percent_key).await?;
        let _: () = conn.del(&hash_key).await?;
        let _: () = conn.hdel("tus:video:ongoing", video_id.to_string()).await?;
        Ok(())
    }

    /// Registers a video as active in Redis.
    pub async fn register_video(&self, video_id: i32) -> Result<()> {
        let mut conn = self.manager.clone();
        let active_key = format!("tus:video:{}:active", video_id);
        let _: () = conn.set(&active_key, true).await?;
        Ok(())
    }

    /// Verifies if a video is active in Redis (returns true if active, false otherwise).
    pub async fn is_video_active(&self, video_id: i32) -> Result<bool> {
        let mut conn = self.manager.clone();
        let active_key = format!("tus:video:{}:active", video_id);
        let exists: bool = conn.exists(active_key).await?;
        Ok(exists)
    }

    /// Deletes the active video key and publishes a cancellation payload to Redis Pub/Sub.
    pub async fn cancel_video(&self, video_id: i32) -> Result<()> {
        let mut conn = self.manager.clone();
        let active_key = format!("tus:video:{}:active", video_id);
        let _: () = conn.del(active_key).await?;
        
        let payload = serde_json::json!({
            "video_id": video_id,
        });
        let _: () = conn.publish("video:transcode:cancel", payload.to_string()).await?;
        Ok(())
    }
}

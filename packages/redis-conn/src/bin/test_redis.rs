use anyhow::Result;
use redis::AsyncCommands;

#[tokio::main]
async fn main() -> Result<()> {
    // env_logger::init();
    println!("Installing default crypto provider...");


    println!("Connecting to Redis...");
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "rediss://default:gQAAAAAAAsAhAAIgcDIzYWIyODM4Y2YxMGE0ODVhOTVjYmU4NmM4MDU0ZTQ2Yw@apparent-horse-180257.upstash.io:6379".to_string());
    
    let client = redis::Client::open(redis_url.clone())?;
    println!("Attempting to establish connection...");
    
    // Use simple async connection instead of ConnectionManager to see the EXACT error
    match client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            println!("Connection successful!");
            let result: redis::RedisResult<Option<String>> = conn.get("tus:video:7:active").await;
            println!("GET tus:video:7:active result: {:?}", result);
        }
        Err(e) => {
            println!("Connection failed: {:?}", e);
        }
    }
    
    Ok(())
}

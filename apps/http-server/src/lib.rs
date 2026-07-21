// lib.rs — re-exports for integration tests.
// The binary (main.rs) is the real entry point; this just makes
// the internal modules accessible to tests/ via `http_server::`.

pub mod errors;
pub mod handlers;
pub mod helpers;
pub mod middleware;

use aws_sdk_s3::Client as S3Client;
use db::DbPool;
use redis_conn::RedisClient;

#[derive(Clone)]
pub struct AppState {
    pub db_pool:   DbPool,
    pub redis:     RedisClient,
    pub s3_client: S3Client,
    pub s3_bucket: String,
}

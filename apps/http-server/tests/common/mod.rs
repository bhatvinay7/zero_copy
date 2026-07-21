//! Integration test helpers shared across all http-server test modules.
//!
//! These tests use `axum-test` to spin up the full router in-process
//! (no real network socket) with a real PostgreSQL + Redis connection
//! pointed at the environment variables set in `.env`.
//!
//! Run:
//!   cargo test -p http-server -- --nocapture

use axum::{
    http::{header, Method},
    Router,
};
use axum_test::TestServer;
use db::{establish_connection_pool, run_migrations};
use rabbitmq_conn::RabbitMQClient;
use redis_conn::RedisClient;
use std::sync::Arc;

use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::Client as S3Client;

// Re-export AppState so individual test modules can construct it
pub use http_server::AppState;

/// Load `.env` once per test run and build a fully wired `AppState`.
/// Each test that needs DB connectivity calls this.
pub async fn build_test_state() -> AppState {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let db_pool = establish_connection_pool(&db_url).expect("DB pool");
    if let Ok(mut conn) = db_pool.get() {
        run_migrations(&mut conn).expect("migrations");
    }

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let redis = RedisClient::new(&redis_url).await.expect("redis");

    let rabbit_url =
        std::env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let rabbitmq = Arc::new(RabbitMQClient::new(&rabbit_url).await.expect("rabbitmq"));

    // Stub S3 client — tests that actually call R2 are gated behind cfg(integration)
    let r2_account_id = std::env::var("CLOUDFLARE_R2_ACCOUNT_ID").unwrap_or_default();
    let r2_access_key = std::env::var("CLOUDFLARE_R2_ACCESS_KEY_ID").unwrap_or_default();
    let r2_secret_key = std::env::var("CLOUDFLARE_R2_SECRET_ACCESS_KEY").unwrap_or_default();
    let r2_bucket = std::env::var("CLOUDFLARE_R2_BUCKET")
        .unwrap_or_else(|_| "test-bucket".into());

    let credentials = Credentials::new(r2_access_key, r2_secret_key, None, None, "test");
    let endpoint = format!("https://{}.r2.cloudflarestorage.com", r2_account_id);
    let cfg = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("auto"))
        .credentials_provider(credentials)
        .endpoint_url(endpoint)
        .load()
        .await;

    AppState {
        db_pool,
        redis,
        rabbitmq,
        s3_client: S3Client::new(&cfg),
        s3_bucket: r2_bucket,
    }
}

/// Build the full Axum router exactly as production `main()` does —
/// minus the Redis pub/sub background task (not needed for unit tests).
pub async fn build_test_router(state: AppState) -> Router {
    use axum::{
        middleware as axum_middleware,
        routing::{get, post},
    };
    use http_server::{
        handlers::{
            auth::{login_handler, signup_handler},
            events::{cancel_video_handler, get_videos_handler, sse_progress_handler},
            r2::{
                abort_multipart_handler, complete_multipart_handler, create_multipart_handler,
                sign_part_handler,
            },
        },
        middleware::auth_guard,
    };
    use tower_http::cors::CorsLayer;

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::HEAD, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .route("/api/auth/signup", post(signup_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/r2/create-multipart", post(create_multipart_handler))
        .route("/api/r2/sign-part", get(sign_part_handler))
        .route("/api/r2/complete-multipart", post(complete_multipart_handler))
        .route("/api/r2/abort-multipart", post(abort_multipart_handler))
        .route("/api/videos/events", get(sse_progress_handler))
        .route("/api/videos/:id/cancel", post(cancel_video_handler))
        .route("/api/videos", get(get_videos_handler))
        .route("/health", get(|| async { "ok" }))
        .route_layer(axum_middleware::from_fn(auth_guard))
        .with_state(state)
        .layer(cors)
}

/// Mint a valid JWT for `user_id` using the same secret as middleware.
pub fn make_bearer_token(user_id: i32) -> String {
    use http_server::middleware::{Claims, JWT_SECRET};
    use jsonwebtoken::{encode, EncodingKey, Header};

    let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize;
    let claims = Claims { sub: user_id, exp };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .expect("encode jwt");
    format!("Bearer {}", token)
}

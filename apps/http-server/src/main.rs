// The actual module definitions live in lib.rs so tests can import them.
// main.rs just pulls them in via `use`.
use http_server::{handlers, middleware, AppState};

use axum::{
    http::{header, Method},
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

// Database and connection pools
use db::{establish_connection_pool, run_migrations};
use redis_conn::RedisClient;

// Handlers and middleware
use handlers::auth::{login_handler, signup_handler};
use handlers::r2::{
    abort_multipart_handler, complete_multipart_handler, create_multipart_handler,
    sign_part_handler,
};
use middleware::auth_guard;

#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting Http Server...");

    // Setup DB
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = establish_connection_pool(&db_url).expect("Failed to connect to PostgreSQL Database");

    // Auto run database table migrations
    if let Ok(mut conn) = db_pool.get() {
        run_migrations(&mut conn).expect("Failed to run database migrations");
    }

    // Setup Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis = RedisClient::new(&redis_url).await.expect("Failed to connect to Redis");

    // Setup S3 Config from s3-conn package
    let s3_config = s3_conn::S3Config::new().await.expect("Failed to initialize S3 config");
    
    let state = AppState {
        db_pool,
        redis,
        s3_client: s3_config.client,
        s3_bucket: s3_config.bucket,
    };

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::HEAD, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
        ]);

    // Setup Routes
    let app = Router::new()
        .route("/api/auth/signup", post(signup_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/health", get(|| async { "ok" }))
        .merge(
            Router::new()
                // S3 multipart APIs (guarded by auth middleware)
                .route("/api/r2/create-multipart", post(create_multipart_handler))
                .route("/api/r2/sign-part", get(sign_part_handler))
                .route("/api/r2/complete-multipart", post(complete_multipart_handler))
                .route("/api/r2/abort-multipart", post(abort_multipart_handler))
                .route("/api/videos/events", get(handlers::events::sse_progress_handler))
                .route("/api/videos/:id/cancel", post(handlers::events::cancel_video_handler))
                .route("/api/videos", get(handlers::events::get_videos_handler))
                .route_layer(axum_middleware::from_fn(auth_guard))
        )
        .with_state(state)
        .layer(cors);

    // Bind to 0.0.0.0 so Docker networking works; port configurable via env
    let port: u16 = std::env::var("HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("HTTP Server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod errors;
mod handlers;
mod helpers;

use axum::{
    http::{header, HeaderName, Method},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use aws_sdk_s3::Client as S3Client;

use db::{establish_connection_pool, run_migrations, DbPool};
use rabbitmq_conn::RabbitMQClient;
use redis_conn::RedisClient;

use handlers::tus::{
    tus_create_handler, tus_delete_handler, tus_head_handler, tus_options_handler,
    tus_patch_handler,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool:   DbPool,
    pub redis:     RedisClient,
    pub rabbitmq:  Arc<RabbitMQClient>,
    pub s3_client: S3Client,
    pub s3_bucket: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    db::load_env();

    log::info!("Starting TUS Server...");
    println!("Starting TUS Server... (println)");

    // ── Database ──────────────────────────────────────────────────────────────
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    println!("DATABASE_URL loaded, establishing connection pool...");
    let db_pool = establish_connection_pool(&db_url)
        .expect("Failed to connect to PostgreSQL");
    println!("DB pool established. Running migrations...");
    if let Ok(mut conn) = db_pool.get() {
        println!("Got DB connection, running migrations...");
        run_migrations(&mut conn).expect("Failed to run DB migrations");
        println!("Migrations completed.");
    } else {
        println!("Failed to get DB connection for migrations.");
    }

    // ── Redis ─────────────────────────────────────────────────────────────────
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    println!("==> TUS-SERVER: Connecting to Redis URL: {}", redis_url);
    let redis = RedisClient::new(&redis_url).await.expect("Failed to connect to Redis");
    println!("Redis connected.");

    // ── RabbitMQ ──────────────────────────────────────────────────────────────
    let rabbit_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".to_string());
    let rabbitmq = Arc::new(RabbitMQClient::new(&rabbit_url));

    // ── Cloudflare R2 ─────────────────────────────────────────────────────────
    let s3_config = s3_conn::S3Config::new().await.expect("Failed to initialize S3 config");

    let state = AppState {
        db_pool,
        redis,
        rabbitmq,
        s3_client: s3_config.client,
        s3_bucket: s3_config.bucket,
    };

    // ── CORS — allow TUS protocol headers ─────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::HEAD,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
        .expose_headers([
            header::LOCATION,
            HeaderName::from_static("upload-offset"),
            HeaderName::from_static("upload-length"),
            HeaderName::from_static("tus-resumable"),
            HeaderName::from_static("tus-version"),
            HeaderName::from_static("tus-extension"),
            HeaderName::from_static("tus-max-size"),
        ]);

    // ── TUS Routes ────────────────────────────────────────────────────────────
    let app = Router::new()
        .route(
            "/files/",
            post(tus_create_handler).options(tus_options_handler),
        )
        .route(
            "/files/:id",
            get(tus_head_handler)
                .head(tus_head_handler)
                .patch(tus_patch_handler)
                .delete(tus_delete_handler),
        )
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
        .layer(cors)
        .layer(axum::extract::DefaultBodyLimit::disable());

    // Bind to 0.0.0.0 so Docker can reach it; port configurable via env
    let port: u16 = std::env::var("TUS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1081);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("TUS Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

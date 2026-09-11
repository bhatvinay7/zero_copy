use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::Response,
    routing::{any, post},
    Router,
};
use reqwest::Client;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    redis: redis_conn::RedisClient,
    rabbitmq: rabbitmq_conn::RabbitMQClient,
    http_client: Client,
    shared_tus_url: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    db::load_env();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis = redis_conn::RedisClient::new_with_pool(&redis_url, 30)
        .await
        .expect("Failed to connect to Redis");

    let rabbit_url =
        std::env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".to_string());
    let rabbitmq = rabbitmq_conn::RabbitMQClient::new(&rabbit_url);

    let shared_tus_url = std::env::var("SHARED_TUS_URL")
        .unwrap_or_else(|_| "http://localhost:1081/files".to_string());

    let state = AppState {
        redis,
        rabbitmq,
        http_client: Client::builder()
            .timeout(Duration::from_secs(3600)) // 1 hour for large files
            .build()
            .unwrap(),
        shared_tus_url,
    };

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
            Method::DELETE,
        ])
        .allow_headers(tower_http::cors::Any)
        .expose_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/files", post(handle_post))
        .route("/files/", post(handle_post))
        .route("/files/:id", any(handle_proxy_existing))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("GATEWAY_PORT").unwrap_or_else(|_| "1080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    log::info!("Gateway listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_user_id(headers: &HeaderMap) -> Option<i32> {
    headers
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
}

async fn handle_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request<Body>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let user_id = match get_user_id(&headers).await {
        Some(id) => id,
        None => return proxy_request(&state.http_client, &state.shared_tus_url, req).await,
    };

    log::info!("POST /files: User {}", user_id);

    // Check subscription in Redis cache
    let sub_key = format!("gateway:user:{}:subscription", user_id);
    let is_premium = if let Ok(Some(sub)) = state.redis.get(&sub_key).await {
        sub == "premium"
    } else {
        false // Default to not premium if not cached
    };
    let target_url = if is_premium {
        resolve_dedicated_pod(&state, user_id).await
    } else {
        state.shared_tus_url.clone()
    };

    // Inject headers
    if is_premium {
        req.headers_mut().insert(
            "X-Dedicated-Queue",
            HeaderValue::from_str(&format!("chunk-queue-premium-{}", user_id)).unwrap(),
        );
    }

    proxy_request(&state.http_client, &target_url, req).await
}

async fn handle_proxy_existing(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let user_id = get_user_id(&headers).await.unwrap_or(0);

    // For PATCH/HEAD, just check Redis for dedicated pod mapping.
    // If not found, use shared pool.
    let mut target_url = state.shared_tus_url.clone();

    if user_id > 0 {
        let redis_key = format!("gateway:user:{}:pod_url", user_id);
        if let Ok(Some(url)) = state.redis.get(&redis_key).await {
            target_url = url;
        }
    }

    let proxy_url = format!("{}/{}", target_url, id);
    proxy_request(&state.http_client, &proxy_url, req).await
}

async fn resolve_dedicated_pod(state: &AppState, user_id: i32) -> String {
    let redis_key = format!("gateway:user:{}:pod_url", user_id);
    let provisioning_key = format!("gateway:user:{}:provisioning", user_id);

    // Check if already mapped
    if let Ok(Some(url)) = state.redis.get(&redis_key).await {
        return url;
    }

    // Check if provisioning
    if let Ok(Some(_)) = state.redis.get(&provisioning_key).await {
        return poll_for_pod(state, &redis_key)
            .await
            .unwrap_or(state.shared_tus_url.clone());
    }

    // Not mapped, not provisioning. Trigger KEDA!
    log::info!(
        "Triggering dedicated KEDA pods for premium user {}",
        user_id
    );
    let _ = state.redis.set(&provisioning_key, "1", Some(60)).await;

    if let Ok(channel) = state.rabbitmq.create_channel().await {
        let payload = serde_json::json!({
            "user_id": user_id,
            "action": "provision",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        let _ = state
            .rabbitmq
            .publish(
                &channel,
                "",
                "keda-provision",
                payload.to_string().as_bytes(),
            )
            .await;
    }

    poll_for_pod(state, &redis_key)
        .await
        .unwrap_or(state.shared_tus_url.clone())
}

async fn poll_for_pod(state: &AppState, redis_key: &str) -> Option<String> {
    log::info!("Polling Redis for dedicated pod at {}...", redis_key);
    for _ in 0..15 {
        sleep(Duration::from_secs(2)).await;
        if let Ok(Some(url)) = state.redis.get(redis_key).await {
            log::info!("Found dedicated pod URL: {}", url);
            return Some(url);
        }
    }
    log::warn!("Timed out waiting for dedicated pod, falling back to shared pool.");
    None
}

async fn proxy_request(
    client: &Client,
    url: &str,
    mut req: Request<Body>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let method = req.method().clone();

    // Remove headers that might mess up proxying
    req.headers_mut().remove(header::HOST);

    let req_builder = client
        .request(method.clone(), url)
        .headers(req.headers().clone())
        .body(reqwest::Body::wrap_stream(
            req.into_body().into_data_stream(),
        ));

    match req_builder.send().await {
        Ok(res) => {
            let mut response = Response::builder().status(res.status());

            // Map headers
            for (k, v) in res.headers() {
                if k != header::TRANSFER_ENCODING
                    && !k.as_str().to_lowercase().starts_with("access-control-")
                {
                    if k == header::LOCATION {
                        // Rewrite Location header to point to the gateway instead of the upstream
                        let mut location_str = v.to_str().unwrap_or("").to_string();
                        // Replace the upstream port/host with our gateway (assuming local testing, or relative path)
                        // Actually, it's safer to just extract the ID and reconstruct the URL relative or using our host.
                        if let Some(idx) = location_str.rfind('/') {
                            let id = &location_str[idx..]; // includes the leading slash, e.g. "/UUID"
                                                           // If we have an absolute URL from frontend, we could use that,
                                                           // but TUS clients generally support relative Location headers.
                            location_str = format!("/files{}", id);
                        }
                        response =
                            response.header(k, HeaderValue::from_str(&location_str).unwrap());
                    } else {
                        response = response.header(k, v);
                    }
                }
            }

            let body = Body::from_stream(res.bytes_stream());
            Ok(response.body(body).unwrap())
        }
        Err(e) => {
            log::error!("Proxy error to {}: {:?}", url, e);
            Err((StatusCode::BAD_GATEWAY, "Bad Gateway".into()))
        }
    }
}

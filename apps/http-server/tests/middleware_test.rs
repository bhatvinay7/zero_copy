//! Tests for the auth middleware (JWT guard)

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;

// ── Auth middleware guard ─────────────────────────────────────────────────────

#[tokio::test]
async fn protected_route_rejects_missing_token() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    // GET /api/videos requires auth
    let res = server.get("/api/videos").await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_rejects_invalid_token() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server
        .get("/api/videos")
        .add_header("Authorization", "Bearer not.a.valid.jwt")
        .await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_accepts_valid_token() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    // Mint a valid token for user_id=1
    let token = common::make_bearer_token(1);

    let res = server
        .get("/api/videos")
        .add_header("Authorization", &token)
        .await;

    // 200 (empty list is fine — no videos yet for this test user)
    assert_eq!(res.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn protected_route_accepts_token_in_query_param() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    // Strip "Bearer " prefix — middleware reads raw token from ?token=
    let token = common::make_bearer_token(1)
        .strip_prefix("Bearer ")
        .unwrap()
        .to_string();

    let res = server
        .get(&format!("/api/videos?token={}", token))
        .await;

    assert_eq!(res.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn health_endpoint_is_public() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server.get("/health").await;
    assert_eq!(res.status_code(), StatusCode::OK);
    assert_eq!(res.text(), "ok");
}

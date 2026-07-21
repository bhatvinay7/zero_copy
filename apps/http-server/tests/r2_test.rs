//! Tests for R2 multipart upload endpoints.
//!
//! These tests exercise route-level validation without calling real R2.
//! Full S3 integration tests are gated behind `#[cfg(feature = "integration")]`.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;

// ── POST /api/r2/create-multipart ─────────────────────────────────────────────

#[tokio::test]
async fn create_multipart_requires_auth() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server
        .post("/api/r2/create-multipart")
        .json(&json!({ "filename": "test.mp4", "content_type": "video/mp4" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_multipart_rejects_missing_body_fields() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();
    let token = common::make_bearer_token(1);

    // Missing content_type
    let res = server
        .post("/api/r2/create-multipart")
        .add_header("Authorization", &token)
        .json(&json!({ "filename": "test.mp4" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── GET /api/r2/sign-part ─────────────────────────────────────────────────────

#[tokio::test]
async fn sign_part_requires_auth() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server
        .get("/api/r2/sign-part?key=uploads/test.mp4&upload_id=abc&part_number=1")
        .await;

    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn sign_part_rejects_missing_query_params() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();
    let token = common::make_bearer_token(1);

    // Missing upload_id and part_number
    let res = server
        .get("/api/r2/sign-part?key=uploads/test.mp4")
        .add_header("Authorization", &token)
        .await;

    // Axum returns 422 for Query extraction failures
    assert_eq!(res.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── POST /api/r2/complete-multipart ───────────────────────────────────────────

#[tokio::test]
async fn complete_multipart_requires_auth() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server
        .post("/api/r2/complete-multipart")
        .json(&json!({
            "key": "uploads/test.mp4",
            "uploadId": "fake-id",
            "parts": [{ "partNumber": 1, "eTag": "\"abc\"" }]
        }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn complete_multipart_rejects_missing_parts() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();
    let token = common::make_bearer_token(1);

    // Missing `parts` field
    let res = server
        .post("/api/r2/complete-multipart")
        .add_header("Authorization", &token)
        .json(&json!({ "key": "uploads/test.mp4", "uploadId": "fake-id" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── POST /api/r2/abort-multipart ──────────────────────────────────────────────

#[tokio::test]
async fn abort_multipart_requires_auth() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server
        .post("/api/r2/abort-multipart")
        .json(&json!({ "key": "uploads/test.mp4", "uploadId": "fake-id" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn abort_multipart_rejects_missing_upload_id() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();
    let token = common::make_bearer_token(1);

    // Missing uploadId
    let res = server
        .post("/api/r2/abort-multipart")
        .add_header("Authorization", &token)
        .json(&json!({ "key": "uploads/test.mp4" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

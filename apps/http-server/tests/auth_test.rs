//! Tests for POST /api/auth/signup and POST /api/auth/login

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};
use uuid::Uuid;

// ── /api/auth/signup ──────────────────────────────────────────────────────────

#[tokio::test]
async fn signup_returns_201_with_new_user() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    // Use a unique email per run to avoid collision with existing rows
    let email = format!("test-signup-{}@example.com", Uuid::new_v4());

    let res = server
        .post("/api/auth/signup")
        .json(&json!({ "email": email, "password": "hunter2" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::CREATED);

    let body: Value = res.json();
    assert_eq!(body["success"], true);
    assert!(body["user"]["id"].is_number());
    assert_eq!(body["user"]["email"], email);
    assert!(body["token"].is_null());
}

#[tokio::test]
async fn signup_rejects_duplicate_email() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let email = format!("dup-{}@example.com", Uuid::new_v4());
    let body = json!({ "email": email, "password": "hunter2" });

    // First signup succeeds
    server.post("/api/auth/signup").json(&body).await;

    // Second signup with same email must fail
    let res = server.post("/api/auth/signup").json(&body).await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);

    let resp: Value = res.json();
    assert_eq!(resp["success"], false);
    assert!(resp["error"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn signup_rejects_missing_fields() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    // Missing password
    let res = server
        .post("/api/auth/signup")
        .json(&json!({ "email": "no-pass@example.com" }))
        .await;
    // Axum returns 422 for deserialization failures
    assert_eq!(res.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── /api/auth/login ───────────────────────────────────────────────────────────

#[tokio::test]
async fn login_returns_200_and_token() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let email = format!("login-{}@example.com", Uuid::new_v4());
    let password = "correct-horse-battery";

    // Register first
    server
        .post("/api/auth/signup")
        .json(&json!({ "email": email, "password": password }))
        .await;

    // Login
    let res = server
        .post("/api/auth/login")
        .json(&json!({ "email": email, "password": password }))
        .await;

    assert_eq!(res.status_code(), StatusCode::OK);

    let body: Value = res.json();
    assert_eq!(body["success"], true);
    assert!(body["token"].is_string());
    assert!(!body["token"].as_str().unwrap().is_empty());
    assert_eq!(body["user"]["email"], email);
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let email = format!("wrongpass-{}@example.com", Uuid::new_v4());
    server
        .post("/api/auth/signup")
        .json(&json!({ "email": email, "password": "correct" }))
        .await;

    let res = server
        .post("/api/auth/login")
        .json(&json!({ "email": email, "password": "wrong" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
    let body: Value = res.json();
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn login_rejects_unknown_email() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server
        .post("/api/auth/login")
        .json(&json!({ "email": "nobody@example.com", "password": "anything" }))
        .await;

    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

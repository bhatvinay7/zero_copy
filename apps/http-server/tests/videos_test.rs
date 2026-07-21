//! Tests for GET /api/videos and POST /api/videos/:id/cancel

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::Value;

// ── GET /api/videos ───────────────────────────────────────────────────────────

#[tokio::test]
async fn get_videos_returns_200_empty_list_for_new_user() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    // Use a high user_id that will have no videos
    let token = common::make_bearer_token(999999);

    let res = server
        .get("/api/videos")
        .add_header("Authorization", &token)
        .await;

    assert_eq!(res.status_code(), StatusCode::OK);
    let body: Value = res.json();
    assert!(body.is_array());
}

#[tokio::test]
async fn get_videos_returns_correct_shape_for_each_video() {
    let state = common::build_test_state().await;
    // Seed a video directly into the DB so we have something to assert on
    let mut conn = state.db_pool.get().expect("db conn");
    let video = db::create_video(
        &mut conn,
        &db::models::NewVideo {
            user_id:      1,
            filename:     "test.mp4".into(),
            original_url: "https://example.com/test.mp4".into(),
            status:       "uploaded".into(),
        },
    )
    .expect("create video");

    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();
    let token = common::make_bearer_token(1);

    let res = server
        .get("/api/videos")
        .add_header("Authorization", &token)
        .await;

    assert_eq!(res.status_code(), StatusCode::OK);

    let body: Vec<Value> = res.json();
    let found = body.iter().find(|v| v["id"] == video.id.to_string());
    assert!(found.is_some(), "seeded video not found in response");

    let v = found.unwrap();
    assert!(v["filename"].is_string());
    assert!(v["status"].is_string());
    assert!(v["progress"].is_number());
    assert!(v["resolutions"].is_array());
    assert!(v["createdAt"].is_string());
}

// ── POST /api/videos/:id/cancel ───────────────────────────────────────────────

#[tokio::test]
async fn cancel_video_returns_200() {
    let state = common::build_test_state().await;
    let mut conn = state.db_pool.get().expect("db conn");
    let video = db::create_video(
        &mut conn,
        &db::models::NewVideo {
            user_id:      1,
            filename:     "cancel-test.mp4".into(),
            original_url: "https://example.com/cancel.mp4".into(),
            status:       "transcoding".into(),
        },
    )
    .expect("create video");

    // Register in Redis so cancel_video doesn't error
    state.redis.register_video(video.id).await.expect("register");

    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();
    let token = common::make_bearer_token(1);

    let res = server
        .post(&format!("/api/videos/{}/cancel", video.id))
        .add_header("Authorization", &token)
        .await;

    assert_eq!(res.status_code(), StatusCode::OK);
    let body: Value = res.json();
    assert_eq!(body["success"], true);
}

#[tokio::test]
async fn cancel_video_requires_auth() {
    let state = common::build_test_state().await;
    let router = common::build_test_router(state).await;
    let server = TestServer::new(router).unwrap();

    let res = server.post("/api/videos/1/cancel").await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

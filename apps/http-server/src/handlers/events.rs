use crate::errors::AppError;
use crate::middleware::Claims;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::{Event, KeepAlive, Sse}, IntoResponse},
    Extension, Json,
};
use futures_util::StreamExt;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub async fn sse_progress_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_id = claims.sub;
    
    // Open async pubsub connection using the raw Redis client
    let client = state.redis.get_client();
    let mut pubsub_conn = client
        .get_async_pubsub()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    pubsub_conn
        .subscribe("video:transcode:progress")
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);

    // Stream incoming pub/sub updates
    tokio::spawn(async move {
        let mut pubsub_stream = pubsub_conn.on_message();
        while let Some(msg) = pubsub_stream.next().await {
            let payload: String = msg.get_payload().unwrap_or_default();
            if let Ok(event_val) = serde_json::from_str::<serde_json::Value>(&payload) {
                if let Some(event_user_id) = event_val.get("user_id").and_then(|v| v.as_i64()) {
                    if event_user_id as i32 == user_id {
                        let sse_event = Event::default()
                            .event("progress")
                            .json_data(&event_val)
                            .unwrap_or_else(|_| Event::default());
                        
                        if tx.send(Ok(sse_event)).await.is_err() {
                            break; // Reader dropped, exit thread
                        }
                    }
                }
            }
        }
        log::info!("Closed SSE progress publisher for user_id: {}", user_id);
    });

    let stream = ReceiverStream::new(rx);

    Ok(Sse::new(stream).keep_alive(KeepAlive::default().interval(Duration::from_secs(15))))
}

pub async fn cancel_video_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(video_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut db_conn = state.db_pool.get().map_err(|e| AppError::Internal(e.into()))?;

    // Update video status to cancelled in the database
    db::update_video_status(&mut db_conn, video_id, "cancelled")?;

    // Trigger video transcode cancellation in Redis and publish PubSub event
    state.redis.cancel_video(video_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Video transcoding job cancelled successfully"
        })),
    ))
}

pub async fn delete_video_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(video_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut db_conn = state.db_pool.get().map_err(|e| AppError::Internal(e.into()))?;

    // 1. Fetch video to verify ownership and get original_url
    let video = db::get_video_by_id(&mut db_conn, video_id)
        .map_err(|_| AppError::NotFound("Video not found".into()))?;

    if video.user_id != claims.sub {
        return Err(AppError::Unauthorized("Not owner".into()));
    }

    // 2. Delete from DB
    db::delete_video_by_id(&mut db_conn, video_id)
        .map_err(|e| AppError::Internal(e.into()))?;

    // 3. Delete original uploaded video from R2 (s3_key is original_url)
    if !video.original_url.is_empty() {
        let _ = state.s3_client.delete_object()
            .bucket(&state.s3_bucket)
            .key(&video.original_url)
            .send()
            .await;
    }

    // 4. We can't use S3Config helper here since we only have raw s3_client,
    // so we'll delete prefixes using pagination directly for chunks, transcoded, and final.
    let prefixes = [
        format!("chunks/{}/", video_id),
        format!("transcoded/{}/", video_id),
        format!("final/{}/", video_id),
    ];

    for prefix in prefixes {
        let mut object_stream = state.s3_client
            .list_objects_v2()
            .bucket(&state.s3_bucket)
            .prefix(&prefix)
            .into_paginator()
            .send();

        while let Some(res) = object_stream.next().await {
            if let Ok(page) = res {
                for obj in page.contents() {
                    if let Some(key) = obj.key() {
                        let _ = state.s3_client.delete_object()
                            .bucket(&state.s3_bucket)
                            .key(key)
                            .send()
                            .await;
                    }
                }
            }
        }
    }

    // 5. Clean up Redis just in case
    let _ = state.redis.cancel_video(video_id).await;
    let _ = state.redis.clean_video_progress(video_id).await;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Video permanently deleted"
        })),
    ))
}


pub async fn get_videos_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let mut db_conn = state.db_pool.get().map_err(|e| AppError::Internal(e.into()))?;

    // Fetch all user videos from database
    let user_id = claims.sub;
    let video_records = db::get_user_videos(&mut db_conn, user_id)?;

    let mut response_list = Vec::new();
    for video in video_records {
        let formats = db::get_video_formats(&mut db_conn, video.id)?;

        // Retrieve current transcode percentage from Redis status tracking keys
        let redis_progress_opt: Option<String> = state.redis.get(&format!("tus:video:{}:last_published_percent", video.id))
            .await
            .unwrap_or(None);

        let progress: u32 = match redis_progress_opt {
            Some(p) => p.parse::<u32>().unwrap_or(0),
            None => {
                if video.status == "completed" {
                    100
                } else {
                    0
                }
            }
        };

        // Fetch detailed per-resolution progress
        let hash_key = format!("tus:video:{}:resolutions", video.id);
        let res_progress: std::collections::HashMap<String, u32> = state.redis.get_hash_all(&hash_key)
            .await
            .unwrap_or_default();

        let mut ui_status = map_db_status_to_ui(&video.status);
        if ui_status == "queued" && progress > 0 {
            ui_status = "processing";
        }

        response_list.push(serde_json::json!({
            "id": video.id.to_string(),
            "filename": video.filename,
            "original_url": video.original_url,
            "status": ui_status,
            "progress": progress,
            "progress_details": res_progress,
            "createdAt": video.created_at,
            "resolutions": formats.into_iter().map(|f| {
                serde_json::json!({
                    "label": format_res_label(&f.resolution),
                    "tag": format_res_tag(&f.resolution),
                    "resolution": f.resolution,
                    "url": f.url,
                })
            }).collect::<Vec<_>>(),
        }));
    }

    Ok((StatusCode::OK, Json(response_list)))
}

fn map_db_status_to_ui(status: &str) -> &'static str {
    match status {
        "uploaded" => "queued",
        "transcoding" => "processing",
        "completed" => "done",
        "cancelled" => "error",
        _ => "queued",
    }
}

fn format_res_label(res: &str) -> &'static str {
    match res {
        "UHD_4K" => "4K UHD",
        "FHD_1080P" => "1080p FHD",
        "HD_720P" => "720p HD",
        "SD_480P" => "480p SD",
        "SD_360P" => "360p",
        _ => "720p HD",
    }
}

fn format_res_tag(res: &str) -> &'static str {
    match res {
        "UHD_4K" => "4K",
        "FHD_1080P" => "1080p",
        "HD_720P" => "720p",
        "SD_480P" => "480p",
        "SD_360P" => "360p",
        _ => "720p",
    }
}

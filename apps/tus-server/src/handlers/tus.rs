use crate::errors::AppError;
use crate::helpers::{get_presigned_url_r2, DateNowMs};
use crate::AppState;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use db::models::NewVideo;
use db::create_video;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum allowed upload size: 10 GiB
const TUS_MAX_SIZE: u64 = 10 * 1024 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompletedPartInput {
    #[serde(rename = "partNumber")]
    pub part_number: i32,
    #[serde(rename = "eTag")]
    pub e_tag: String,
}

/// TUS session metadata persisted in Redis
#[derive(Serialize, Deserialize, Clone)]
pub struct TusMetadata {
    pub s3_key:         String,
    pub upload_id:      String,
    pub filename:       String,
    pub total_length:   u64,
    pub current_offset: u64,
    pub parts:          Vec<CompletedPartInput>,
    pub user_id:        i32,
}

// ── OPTIONS /files/ ───────────────────────────────────────────────────────────

pub async fn tus_options_handler() -> impl IntoResponse {
    let mut h = HeaderMap::new();
    h.insert("Tus-Resumable", HeaderValue::from_static("1.0.0"));
    h.insert("Tus-Version",   HeaderValue::from_static("1.0.0"));
    h.insert("Tus-Extension", HeaderValue::from_static("creation,creation-with-upload,termination"));
    h.insert("Tus-Max-Size",  HeaderValue::from_str(&TUS_MAX_SIZE.to_string()).unwrap());
    (StatusCode::NO_CONTENT, h)
}

// ── POST /files/ ──────────────────────────────────────────────────────────────

pub async fn tus_create_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    log::info!("Starting POST /files/...");
    require_tus_version(&headers)?;

    let total_length: u64 = headers
        .get("upload-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("Missing upload-length".into()))?;

    if total_length > TUS_MAX_SIZE {
        return Err(AppError::BadRequest(format!(
            "Upload size {total_length} exceeds max {TUS_MAX_SIZE}"
        )));
    }

    let (filename, user_id) = parse_upload_metadata(
        headers.get("upload-metadata").and_then(|v| v.to_str().ok()).unwrap_or(""),
    );

    let safe = sanitize_filename(&filename);
    let s3_key = format!("uploads/{}-{}-{}", Uuid::new_v4(), DateNowMs(), safe);

    // Create S3 multipart upload in Cloudflare R2
    log::info!("POST: Before S3 create_multipart_upload...");
    let s3_res = state
        .s3_client
        .create_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .content_type("video/mp4")
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    log::info!("POST: After S3 create_multipart_upload.");

    let upload_id = s3_res
        .upload_id()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing upload ID")))?
        .to_string();

    let tus_id = Uuid::new_v4().to_string();
    println!("==> TUS: Created multipart upload! tus_id={} upload_id={}", tus_id, upload_id);
    let meta = TusMetadata {
        s3_key,
        upload_id,
        filename,
        total_length,
        current_offset: 0,
        parts: vec![],
        user_id,
    };

    persist_meta(&state, &tus_id, &meta).await?;

    // Construct the absolute URL using the Host header (which Nginx preserves via $http_host)
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:1081");
        
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let location = format!("{}://{}/files/{}", proto, host, tus_id);

    let mut h = HeaderMap::new();
    h.insert("Tus-Resumable", HeaderValue::from_static("1.0.0"));
    h.insert(header::LOCATION, HeaderValue::from_str(&location).unwrap());
    Ok((StatusCode::CREATED, h))
}

// ── HEAD /files/:id ───────────────────────────────────────────────────────────

pub async fn tus_head_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    require_tus_version(&headers)?;
    let meta = load_meta(&state, &id).await?;

    let mut h = HeaderMap::new();
    h.insert("Tus-Resumable",  HeaderValue::from_static("1.0.0"));
    h.insert("Upload-Offset",  HeaderValue::from_str(&meta.current_offset.to_string()).unwrap());
    h.insert("Upload-Length",  HeaderValue::from_str(&meta.total_length.to_string()).unwrap());
    h.insert("Cache-Control",  HeaderValue::from_static("no-store"));
    Ok((StatusCode::OK, h))
}

// ── PATCH /files/:id ─────────────────────────────────────────────────────────

pub async fn tus_patch_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    require_tus_version(&headers)?;

    let req_offset: u64 = headers
        .get("upload-offset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("Missing upload-offset".into()))?;

    log::info!("PATCH {}: Starting load_meta...", id);
    let mut meta = load_meta(&state, &id).await?;
    log::info!("PATCH {}: load_meta successful.", id);

    if req_offset != meta.current_offset {
        return Err(AppError::Conflict("Offset mismatch".into()));
    }
    if body.is_empty() {
        return Err(AppError::BadRequest("Empty body".into()));
    }

    // Upload chunk as an S3 multipart part
    let part_number = (meta.parts.len() + 1) as i32;
    println!("==> TUS: Calling upload_part for tus_id={} upload_id={} part={} chunk_size={}", id, meta.upload_id, part_number, body.len());
    let part_res = state
        .s3_client
        .upload_part()
        .bucket(&state.s3_bucket)
        .key(&meta.s3_key)
        .upload_id(&meta.upload_id)
        .part_number(part_number)
        .body(aws_sdk_s3::primitives::ByteStream::from(body.clone()))
        .send()
        .await
        .map_err(|e| {
            println!("==> TUS: upload_part FAILED for tus_id={} upload_id={}: {:?}", id, meta.upload_id, e);
            AppError::Internal(e.into())
        })?;
    println!("==> TUS: S3 upload_part successful for tus_id={}", id);

    let e_tag = part_res
        .e_tag()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing ETag")))?
        .to_string();

    meta.current_offset += body.len() as u64;
    meta.parts.push(CompletedPartInput { part_number, e_tag });

    // Save offset before potentially moving `meta` into on_upload_complete
    let new_offset = meta.current_offset;

    if new_offset >= meta.total_length {
        on_upload_complete(&state, &id, meta).await?;
    } else {
        persist_meta(&state, &id, &meta).await?;
    }

    let mut h = HeaderMap::new();
    h.insert("Tus-Resumable", HeaderValue::from_static("1.0.0"));
    h.insert("Upload-Offset", HeaderValue::from_str(&new_offset.to_string()).unwrap());
    Ok((StatusCode::NO_CONTENT, h))
}

// ── DELETE /files/:id — termination extension ─────────────────────────────────

pub async fn tus_delete_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    require_tus_version(&headers)?;
    let meta = load_meta(&state, &id).await?;

    println!("==> TUS: DELETE handler called for tus_id={} upload_id={}", id, meta.upload_id);
    // Abort the S3 multipart upload to free R2 storage
    let res = state
        .s3_client
        .abort_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(&meta.s3_key)
        .upload_id(&meta.upload_id)
        .send()
        .await;
    println!("==> TUS: abort_multipart_upload result for tus_id={}: {:?}", id, res);

    let redis_key = format!("tus:upload:{}", id);
    let _ = state.redis.del(&redis_key).await;

    let mut h = HeaderMap::new();
    h.insert("Tus-Resumable", HeaderValue::from_static("1.0.0"));
    Ok((StatusCode::NO_CONTENT, h))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn require_tus_version(headers: &HeaderMap) -> Result<(), AppError> {
    match headers.get("Tus-Resumable").and_then(|v| v.to_str().ok()) {
        Some("1.0.0") => Ok(()),
        _ => Err(AppError::BadRequest("Invalid or missing Tus-Resumable header".into())),
    }
}

fn parse_upload_metadata(raw: &str) -> (String, i32) {
    let mut filename = "upload.mp4".to_string();
    let mut user_id: i32 = 1;

    for part in raw.split(',') {
        let part = part.trim();
        let mut kv = part.splitn(2, ' ');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("");
        let decoded = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, val)
            .unwrap_or_default();
        let s = String::from_utf8(decoded).unwrap_or_default();
        match key {
            "filename" => filename = s,
            "userId"   => user_id = s.parse().unwrap_or(1),
            _          => {}
        }
    }
    (filename, user_id)
}

fn sanitize_filename(name: &str) -> String {
    name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_', "_")
}

async fn persist_meta(state: &AppState, id: &str, meta: &TusMetadata) -> Result<(), AppError> {
    let key = format!("tus:upload:{}", id);
    let json = serde_json::to_string(meta).map_err(|e| AppError::Internal(e.into()))?;
    state
        .redis
        .set(&key, &json, Some(86400))
        .await
        .map_err(|e| AppError::Internal(e.into()))
}

async fn load_meta(state: &AppState, id: &str) -> Result<TusMetadata, AppError> {
    let key = format!("tus:upload:{}", id);
    let json = state
        .redis
        .get(&key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound("Upload session not found".into()))?;
    serde_json::from_str(&json).map_err(|e| AppError::Internal(e.into()))
}

/// Called when the final chunk is received: completes S3 multipart, creates DB
/// record, and pushes a transcode job to RabbitMQ.
async fn on_upload_complete(
    state: &AppState,
    id: &str,
    meta: TusMetadata,
) -> Result<(), AppError> {
    // Complete the S3 multipart upload
    let parts: Vec<CompletedPart> = meta
        .parts
        .iter()
        .map(|p| {
            CompletedPart::builder()
                .part_number(p.part_number)
                .e_tag(&p.e_tag)
                .build()
        })
        .collect();

    println!("==> TUS: Calling complete_multipart_upload for tus_id={} upload_id={}", id, meta.upload_id);
    state
        .s3_client
        .complete_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(&meta.s3_key)
        .upload_id(&meta.upload_id)
        .multipart_upload(CompletedMultipartUpload::builder().set_parts(Some(parts)).build())
        .send()
        .await
        .map_err(|e| {
            println!("==> TUS: complete_multipart_upload FAILED for tus_id={} upload_id={}: {:?}", id, meta.upload_id, e);
            AppError::Internal(e.into())
        })?;
    println!("==> TUS: complete_multipart_upload SUCCESSFUL for tus_id={}", id);

    // Presigned 24h download URL
    let download_url = get_presigned_url_r2(
        &state.s3_client,
        &state.s3_bucket,
        &meta.s3_key,
        std::time::Duration::from_secs(86400),
    )
    .await
    .map_err(|e| AppError::Internal(e))?;

    // Create video record in PostgreSQL
    let mut db_conn = state.db_pool.get().map_err(|e| AppError::Internal(e.into()))?;
    let video = create_video(
        &mut db_conn,
        &NewVideo {
            user_id:      meta.user_id,
            filename:     meta.filename.clone(),
            original_url: download_url.clone(),
            status:       "uploaded".to_string(),
        },
    )?;

    // Register video as active in Redis
    state
        .redis
        .register_video(video.id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // Push transcode job to RabbitMQ `chunk-queue`
    let payload = db::models::ChunkJob {
        video_id:     video.id,
        user_id:      video.user_id,
        filename:     meta.filename.clone(),
        s3_key:       meta.s3_key.clone(),
        download_url: download_url,
        size:         meta.total_length,
    };
    let payload = serde_json::to_value(&payload).unwrap();
    if let Ok(channel) = state.rabbitmq.create_channel().await {
        let _ = state
            .rabbitmq
            .publish(&channel, "", "chunk-queue", payload.to_string().as_bytes())
            .await;
    }

    // Publish completion event and clean up Redis session
    let _ = state.redis.publish("tus:complete", &meta.s3_key).await;
    let _ = state.redis.del(&format!("tus:upload:{}", id)).await;

    log::info!("Upload complete: video_id={}, file={}", video.id, meta.filename);
    Ok(())
}

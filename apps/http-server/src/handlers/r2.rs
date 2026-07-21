use crate::errors::AppError;
use crate::helpers::DateNowMs;
use crate::AppState;
use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, Json};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateMultipartReq {
    pub filename: String,
    pub content_type: String,
}

#[derive(Serialize)]
pub struct CreateMultipartRes {
    pub upload_id: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct SignPartQuery {
    pub key: String,
    pub upload_id: String,
    pub part_number: i32,
}

#[derive(Serialize)]
pub struct SignPartRes {
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompletedPartInput {
    #[serde(rename = "partNumber")]
    pub part_number: i32,
    #[serde(rename = "eTag")]
    pub e_tag: String,
}

#[derive(Deserialize)]
pub struct CompleteMultipartReq {
    pub key: String,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    pub parts: Vec<CompletedPartInput>,
}

#[derive(Deserialize)]
pub struct AbortMultipartReq {
    pub key: String,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
}

pub async fn create_multipart_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateMultipartReq>,
) -> Result<impl IntoResponse, AppError> {
    let safe_filename = payload.filename.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_', "_");
    let key = format!("uploads/{}-{}-{}", Uuid::new_v4(), DateNowMs(), safe_filename);

    let res = state.s3_client
        .create_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(&key)
        .content_type(payload.content_type)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let upload_id = res.upload_id()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("R2 failed to return an upload ID")))?;

    Ok(Json(CreateMultipartRes {
        upload_id: upload_id.to_string(),
        key,
    }))
}

pub async fn sign_part_handler(
    State(state): State<AppState>,
    Query(query): Query<SignPartQuery>,
) -> Result<impl IntoResponse, AppError> {
    let presign_config = PresigningConfig::expires_in(std::time::Duration::from_secs(3600))
        .map_err(|e| AppError::Internal(anyhow::Error::new(e)))?;

    let presigned = state.s3_client
        .upload_part()
        .bucket(&state.s3_bucket)
        .key(&query.key)
        .upload_id(&query.upload_id)
        .part_number(query.part_number)
        .presigned(presign_config)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(SignPartRes {
        url: presigned.uri().to_string(),
    }))
}

pub async fn complete_multipart_handler(
    State(state): State<AppState>,
    Json(payload): Json<CompleteMultipartReq>,
) -> Result<impl IntoResponse, AppError> {
    let mut completed_parts = Vec::new();
    for p in payload.parts {
        completed_parts.push(
            CompletedPart::builder()
                .part_number(p.part_number)
                .e_tag(p.e_tag)
                .build(),
        );
    }

    let completed_upload = CompletedMultipartUpload::builder()
        .set_parts(Some(completed_parts))
        .build();

    state.s3_client
        .complete_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(&payload.key)
        .upload_id(&payload.upload_id)
        .multipart_upload(completed_upload)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(StatusCode::OK)
}

pub async fn abort_multipart_handler(
    State(state): State<AppState>,
    Json(payload): Json<AbortMultipartReq>,
) -> Result<impl IntoResponse, AppError> {
    state.s3_client
        .abort_multipart_upload()
        .bucket(&state.s3_bucket)
        .key(&payload.key)
        .upload_id(&payload.upload_id)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(StatusCode::OK)
}

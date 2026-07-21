use crate::schema::{users, video_formats, videos};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id:            i32,
    pub email:         String,
    pub password_hash: String,
    pub created_at:    DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email:         String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    Uhd4k,
    Fhd1080p,
    Hd720p,
    Sd480p,
    Sd360p,
}

impl Resolution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uhd4k    => "UHD_4K",
            Self::Fhd1080p => "FHD_1080P",
            Self::Hd720p   => "HD_720P",
            Self::Sd480p   => "SD_480P",
            Self::Sd360p   => "SD_360P",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "UHD_4K"    => Some(Self::Uhd4k),
            "FHD_1080P" => Some(Self::Fhd1080p),
            "HD_720P"   => Some(Self::Hd720p),
            "SD_480P"   => Some(Self::Sd480p),
            "SD_360P"   => Some(Self::Sd360p),
            _           => None,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = videos)]
pub struct Video {
    pub id:           i32,
    pub user_id:      i32,
    pub filename:     String,
    pub original_url: String,
    pub status:       String,
    pub created_at:   DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone, Deserialize)]
#[diesel(table_name = videos)]
pub struct NewVideo {
    pub user_id:      i32,
    pub filename:     String,
    pub original_url: String,
    pub status:       String,
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = video_formats)]
pub struct VideoFormat {
    pub id:         i32,
    pub video_id:   i32,
    pub resolution: String,
    pub url:        String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone, Deserialize)]
#[diesel(table_name = video_formats)]
pub struct NewVideoFormat {
    pub video_id:   i32,
    pub resolution: String,
    pub url:        String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkJob {
    pub video_id: i32,
    pub user_id: i32,
    pub filename: String,
    pub s3_key: String,
    pub download_url: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranscodeJob {
    pub video_id: i32,
    pub user_id: i32,
    pub chunk_index: u32,
    pub chunk_url: String,
    pub resolution: String,
    pub current_chunk: u32,
    pub total_chunks: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbUpdateMessage {
    pub video_id: i32,
    pub user_id: i32,
    pub chunk_index: u32,
    pub resolution: String,
    pub transcoded_url: String,
    pub current_chunk: u32,
    pub total_chunks: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DbUpdateEvent {
    ProcessingStart {
        video_id: i32,
        user_id: i32,
    },
    ChunkComplete(DbUpdateMessage),
    MergeComplete {
        video_id: i32,
        user_id: i32,
        resolution: String,
        url: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MergeJob {
    pub video_id: i32,
    pub user_id: i32,
    pub resolution: String,
    pub total_chunks: u32,
}

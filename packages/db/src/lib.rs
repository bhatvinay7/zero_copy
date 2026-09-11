pub mod models;
pub mod schema;

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use models::{NewUser, NewVideo, NewVideoFormat, User, Video, VideoFormat};
use schema::users::dsl::{id as u_id, email, users};
use schema::video_formats::dsl::{video_formats, video_id as vf_vid};
use schema::videos::dsl::{id as v_id, status as v_status, videos};

/// Load environment variables from .env file
pub fn load_env() {
    dotenvy::dotenv().ok();
}

/// Embedded Diesel migrations compiled into the binary at build time.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Build an r2d2 connection pool for the Neon PostgreSQL database.
pub fn establish_connection_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .context("Failed to create database connection pool")?;
    Ok(pool)
}

/// Run any pending Diesel migrations. Safe to call on every startup.
pub fn run_migrations(conn: &mut PgConnection) -> Result<()> {
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Failed to run database migrations: {e}"))?;
    Ok(())
}

pub fn get_user_by_email(conn: &mut PgConnection, user_email: &str) -> Result<Option<User>> {
    let user = users
        .filter(email.eq(user_email))
        .first::<User>(conn)
        .optional()
        .context("Error querying user by email")?;
    Ok(user)
}

pub fn get_user_by_id(conn: &mut PgConnection, user_id: i32) -> Result<User> {
    let user = users
        .filter(u_id.eq(user_id))
        .first::<User>(conn)
        .context("Error querying user by id")?;
    Ok(user)
}

pub fn create_user(conn: &mut PgConnection, new_user: &NewUser) -> Result<User> {
    let created = diesel::insert_into(users)
        .values(new_user)
        .get_result::<User>(conn)
        .context("Error inserting new user")?;
    Ok(created)
}

pub fn create_video(conn: &mut PgConnection, new_video: &NewVideo) -> Result<Video> {
    let created = diesel::insert_into(videos)
        .values(new_video)
        .get_result::<Video>(conn)
        .context("Error inserting new video record")?;
    Ok(created)
}

pub fn get_video_by_id(conn: &mut PgConnection, video_id: i32) -> Result<Video> {
    let video = videos
        .filter(v_id.eq(video_id))
        .first::<Video>(conn)
        .context("Error fetching video by id")?;
    Ok(video)
}

pub fn delete_video_by_id(conn: &mut PgConnection, video_id: i32) -> Result<()> {
    diesel::delete(videos.filter(v_id.eq(video_id)))
        .execute(conn)
        .context("Error deleting video")?;
    Ok(())
}

pub fn update_video_status(
    conn: &mut PgConnection,
    video_id: i32,
    new_status: &str,
) -> Result<Video> {
    let updated = diesel::update(videos.filter(v_id.eq(video_id)))
        .set(v_status.eq(new_status))
        .get_result::<Video>(conn)
        .context("Error updating video status")?;
    Ok(updated)
}

pub fn add_video_format(
    conn: &mut PgConnection,
    new_format: &NewVideoFormat,
) -> Result<VideoFormat> {
    let created = diesel::insert_into(video_formats)
        .values(new_format)
        .get_result::<VideoFormat>(conn)
        .context("Error inserting video format record")?;
    Ok(created)
}

pub fn get_video_formats(conn: &mut PgConnection, video_id_val: i32) -> Result<Vec<VideoFormat>> {
    let results = video_formats
        .filter(vf_vid.eq(video_id_val))
        .load::<VideoFormat>(conn)
        .context("Error loading video formats")?;
    Ok(results)
}

pub fn get_user_videos(conn: &mut PgConnection, user_id_val: i32) -> Result<Vec<Video>> {
    use schema::videos::dsl::{user_id as v_uid, videos};
    let results = videos
        .filter(v_uid.eq(user_id_val))
        .order(v_id.desc())
        .load::<Video>(conn)
        .context("Error loading user videos")?;
    Ok(results)
}

use anyhow::Result;
use db::{establish_connection_pool, run_migrations};

fn main() -> Result<()> {
    // Load .env file if present (useful for local development)
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("🔗 Connecting to database...");
    let pool = establish_connection_pool(&database_url)?;
    let mut conn = pool.get()
        .map_err(|e| anyhow::anyhow!("Failed to get connection from pool: {e}"))?;

    println!("🚀 Running pending migrations...");
    run_migrations(&mut conn)?;

    println!("✅ Migrations applied successfully.");
    Ok(())
}

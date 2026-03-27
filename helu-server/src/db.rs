use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use anyhow::{Context, Result};
use tracing::info;

pub async fn setup_db(db_url: &str, max_connections: u32) -> Result<PgPool> {
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(db_url)
        .await
        .context("Failed to connect to the database")?;

    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    info!("Database setup complete.");
    Ok(pool)
}

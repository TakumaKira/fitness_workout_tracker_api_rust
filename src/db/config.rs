use sqlx::{Pool, Any, AnyPool, any};
use std::env;

/// Default SQLite database URL for development purposes only.
/// This creates an auto-generated SQLite database file 'dev.db' in the current directory.
/// @remarks Do not use this in production environments.
const DEFAULT_DATABASE_URL: &str = "sqlite://dev.db?mode=rwc";

pub async fn create_pool() -> Result<Pool<Any>, sqlx::Error> {
    // Try to load from .env file, ignore if it doesn't exist
    let _ = dotenvy::dotenv();

    // Default to SQLite for development
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

    any::install_default_drivers();

    AnyPool::connect(&database_url).await
} 
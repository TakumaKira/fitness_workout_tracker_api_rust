use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{SqliteConnection, PgConnection};
use std::env;

#[derive(Clone)]
enum DbConnection {
    Sqlite(Pool<ConnectionManager<SqliteConnection>>),
    Postgres(Pool<ConnectionManager<PgConnection>>),
}

#[derive(Clone)]
pub struct DatabaseConnection {
    connection: DbConnection,
}

/// Default SQLite database URL for development purposes only.
/// This creates an auto-generated SQLite database file 'dev.db' in the current directory.
/// @remarks Do not use this in production environments.
const DEFAULT_DATABASE_URL: &str = "sqlite://dev.db";

pub fn create_connection() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

    let connection = if database_url.starts_with("postgres://") {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        DbConnection::Postgres(
            Pool::builder()
                .build(manager)
                .expect("Failed to create postgres pool")
        )
    } else {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        DbConnection::Sqlite(
            Pool::builder()
                .build(manager)
                .expect("Failed to create sqlite pool")
        )
    };

    DatabaseConnection { connection }
}

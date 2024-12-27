# Fitness Workout Tracker API Rust

This is a challenge from [here](https://roadmap.sh/backend/project-ideas#9-fitness-workout-tracker).

To run, you can simply run `cargo run` in the terminal, but I recommend using `cargo watch -x 'run'` for development. You need [cargo-watch](https://crates.io/crates/cargo-watch) installed.

## Database

### Specify the database URL

The database is a SQLite database that is created in the `src/db/config.rs` file by default. You can change the database URL in the `.env` file by setting the `DATABASE_URL` variable.

### Database Migrations

```bash
# Install diesel CLI if you haven't already
cargo install diesel_cli --no-default-features --features sqlite,postgres

# Generate a new migration named 'create_users' for development (SQLite)
diesel migration generate <MIGRATION_NAME> --diff-schema --migration-dir migrations/sqlite --database-url sqlite://dev.db

# Generate a new migration named 'create_users' for production (PostgreSQL)
diesel migration generate <MIGRATION_NAME> --diff-schema --migration-dir migrations/postgres --database-url <DATABASE_URL>

# Run the migrations for development (SQLite)
diesel migration run --migration-dir migrations/sqlite --database-url sqlite://dev.db

# Or, run the migrations for production (currently, PostgreSQL is supported)
diesel migration run --migration-dir migrations/postgres --database-url <DATABASE_URL>

# If you need to revert on development (SQLite)
diesel migration revert --database-url sqlite://dev.db

# Or, revert on production (PostgreSQL)
diesel migration revert --database-url <DATABASE_URL>
```

## Environment Variables

The `PORT` variable is used to set the port that the server will run on. You can change the port in the `.env` file by setting the `PORT` variable.
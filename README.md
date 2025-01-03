# Fitness Workout Tracker API Rust

This is a challenge from [here](https://roadmap.sh/backend/project-ideas#9-fitness-workout-tracker).

To run, you can simply run

```bash
cargo run
```

in the terminal, but I recommend using

```bash
cargo watch -x 'run'
```

for development. But, you need [cargo-watch](https://crates.io/crates/cargo-watch) installed.

## Database

### Specify the database URL

The database should be a PostgreSQL database. You need to have a database created with the name `fitness_tracker_api_rust` and the database URL in the `.env` file by setting the `DATABASE_URL` variable.

* Note: I wanted to use SQLite for development, but currently popular ORMs for Rust don't support dynamic switching database engines. I prioritize PostgreSQL for production.

### Database Migrations

```bash
# Install diesel CLI if you haven't already
cargo install diesel_cli --no-default-features --features postgres

# Generate a new migration named 'create_users' for production (PostgreSQL)
diesel migration generate <MIGRATION_NAME> --diff-schema --database-url <DATABASE_URL>

# Run the migrations for production (currently, PostgreSQL is supported)
diesel migration run --database-url <DATABASE_URL>

# If you need to revert on production (PostgreSQL)
diesel migration revert --database-url <DATABASE_URL>

# After generating a migration, you need to update the schema.rs file
diesel print-schema --database-url <DATABASE_URL> > src/schema.rs
# If the result is not what you expect, make sure the schema name and database name is matching in the diesel.toml file and URL
```

## Environment Variables

The `PORT` variable is used to set the port that the server will run on. You can change the port in the `.env` file by setting the `PORT` variable.

## Testing

### Unit Tests

```bash
cargo test
```

### E2E Tests

```bash
scripts/run_postman_tests.sh
```

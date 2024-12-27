# Fitness Workout Tracker API Rust

This is a challenge from [here](https://roadmap.sh/backend/project-ideas#9-fitness-workout-tracker).

To run, you can simply run `cargo run` in the terminal, but I recommend using `cargo watch -x 'run'` for development. You need [cargo-watch](https://crates.io/crates/cargo-watch) installed.

## Database

The database is a SQLite database that is created in the `src/db/config.rs` file by default. You can change the database URL in the `.env` file by setting the `DATABASE_URL` variable.

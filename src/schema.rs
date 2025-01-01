// @generated automatically by Diesel CLI.

pub mod public {
    diesel::table! {
        exercises (id) {
            id -> Int8,
            uuid -> Uuid,
            user_id -> Int8,
            name -> Varchar,
            description -> Nullable<Varchar>,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        sessions (id) {
            id -> Int8,
            user_id -> Int8,
            token -> Varchar,
            csrf_token -> Varchar,
            expires_at -> Timestamp,
            created_at -> Timestamp,
        }
    }

    diesel::table! {
        temp_sessions (id) {
            id -> Int8,
            session_id -> Varchar,
            csrf_token -> Varchar,
            created_at -> Timestamp,
            expires_at -> Timestamp,
        }
    }

    diesel::table! {
        users (id) {
            id -> Int8,
            uuid -> Uuid,
            email -> Varchar,
            password_hash -> Varchar,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        workout_exercises (workout_id, exercise_id) {
            workout_id -> Int8,
            exercise_id -> Int8,
            user_id -> Int8,
            order -> Int4,
        }
    }

    diesel::table! {
        workouts (id) {
            id -> Int8,
            uuid -> Uuid,
            user_id -> Int8,
            name -> Varchar,
            description -> Nullable<Text>,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    diesel::joinable!(exercises -> users (user_id));
    diesel::joinable!(sessions -> users (user_id));
    diesel::joinable!(workout_exercises -> exercises (exercise_id));
    diesel::joinable!(workout_exercises -> users (user_id));
    diesel::joinable!(workout_exercises -> workouts (workout_id));
    diesel::joinable!(workouts -> users (user_id));

    diesel::allow_tables_to_appear_in_same_query!(
        exercises,
        sessions,
        temp_sessions,
        users,
        workout_exercises,
        workouts,
    );
}

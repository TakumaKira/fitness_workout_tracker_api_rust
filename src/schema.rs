// @generated automatically by Diesel CLI.

pub mod public {
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

    diesel::joinable!(sessions -> users (user_id));

    diesel::allow_tables_to_appear_in_same_query!(
        sessions,
        temp_sessions,
        users,
    );
}

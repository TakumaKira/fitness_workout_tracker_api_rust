// @generated automatically by Diesel CLI.

pub mod public {
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
}

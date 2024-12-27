use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::public::sessions)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub csrf_token: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::public::sessions)]
pub struct NewSession {
    pub user_id: i64,
    pub token: String,
    pub csrf_token: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl Session {
    pub fn new(user_id: i64, session_id: String, csrf_token: String) -> NewSession {
        let now = chrono::Utc::now().naive_utc();
        NewSession {
            user_id,
            token: session_id,
            csrf_token,
            expires_at: now + chrono::Duration::hours(24),
            created_at: now,
        }
    }
} 
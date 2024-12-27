use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::public::sessions)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub csrf_token: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl Session {
    pub fn new(user_id: i64, csrf_token: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: 0, // Will be set by trigger
            user_id,
            token: uuid::Uuid::new_v4().to_string(),
            csrf_token,
            expires_at: now + chrono::Duration::hours(24),
            created_at: now,
        }
    }
} 
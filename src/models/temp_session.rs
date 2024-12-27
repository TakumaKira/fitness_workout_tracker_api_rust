use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::public::temp_sessions)]
pub struct TempSession {
    pub id: i64,
    pub session_id: String,
    pub csrf_token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

impl TempSession {
    pub fn new(session_id: String, csrf_token: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: 0,
            session_id,
            csrf_token,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5), // 5 minutes expiration
        }
    }
} 
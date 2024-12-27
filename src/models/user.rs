use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::public::users)]
pub struct User {
    pub id: i64,
    pub uuid: uuid::Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
  #[diesel(table_name = crate::schema::public::users)]
pub struct NewUser {
    pub uuid: uuid::Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn new(email: String, password_hash: String) -> NewUser {
        let now = chrono::Utc::now().naive_utc();
        NewUser {
            uuid: Uuid::new_v4(),
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
} 
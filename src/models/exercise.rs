use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[diesel(table_name = crate::schema::public::exercises)]
pub struct Exercise {
    pub id: i64,
    pub uuid: Uuid,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::public::exercises)]
pub struct NewExercise {
    pub uuid: Uuid,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateExercise {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExercise {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Exercise {
    pub fn new(user_id: i64, name: String, description: Option<String>) -> NewExercise {
        let now = chrono::Utc::now().naive_utc();
        NewExercise {
            uuid: Uuid::new_v4(),
            user_id,
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
} 
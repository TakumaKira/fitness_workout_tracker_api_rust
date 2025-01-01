use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[diesel(table_name = crate::schema::public::workouts)]
pub struct Workout {
    pub id: i64,
    pub uuid: uuid::Uuid,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::public::workouts)]
pub struct NewWorkout {
    pub uuid: Uuid,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Workout {
    pub fn new(user_id: i64, name: String, description: Option<String>) -> NewWorkout {
        let now = chrono::Utc::now().naive_utc();
        NewWorkout {
            uuid: Uuid::new_v4(),
            user_id,
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkout {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkout {
    pub name: Option<String>,
    pub description: Option<String>,
}
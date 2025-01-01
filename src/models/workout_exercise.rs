use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::schema::public::workout_exercises)]
pub struct WorkoutExercise {
    pub workout_id: i64,
    pub exercise_id: i64,
    pub user_id: i64,
    pub order: i32,
}

#[derive(Deserialize)]
pub struct AddExerciseRequest {
    pub exercise_uuid: Uuid,
    pub order: i32,
}
use diesel::prelude::*;
use uuid::Uuid;
use crate::{db, models::{exercise::Exercise, workout_exercise::WorkoutExercise}};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum WorkoutExerciseError {
    DatabaseError(String),
    NotFound,
    WorkoutNotFound,
    ExerciseNotFound,
    Unauthorized,
}

impl From<diesel::result::Error> for WorkoutExerciseError {
    fn from(err: diesel::result::Error) -> WorkoutExerciseError {
        match err {
            diesel::result::Error::NotFound => {
                if err.to_string().contains("workouts") {
                    WorkoutExerciseError::WorkoutNotFound
                } else if err.to_string().contains("exercises") {
                    WorkoutExerciseError::ExerciseNotFound
                } else {
                    WorkoutExerciseError::NotFound
                }
            },
            _ => WorkoutExerciseError::DatabaseError(err.to_string()),
        }
    }
}

pub trait WorkoutExerciseRepository {
    fn add_exercise_to_workout(&self, user_id: i64, workout_uuid: Uuid, exercise_uuid: Uuid, order: i32) -> Result<(), WorkoutExerciseError>;
    fn remove_exercise_from_workout(&self, user_id: i64, workout_uuid: Uuid, exercise_uuid: Uuid) -> Result<(), WorkoutExerciseError>;
    fn list_workout_exercises(&self, user_id: i64, workout_uuid: Uuid) -> Result<Vec<(Exercise, i32)>, WorkoutExerciseError>;
}

pub struct PgWorkoutExerciseRepository;

impl PgWorkoutExerciseRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl WorkoutExerciseRepository for PgWorkoutExerciseRepository {
    fn add_exercise_to_workout(&self, user_id: i64, workout_uuid: Uuid, exercise_uuid: Uuid, order: i32) -> Result<(), WorkoutExerciseError> {
        use crate::schema::public::{exercises, workouts, workout_exercises};
        let mut conn = db::config::establish_connection();

        let workout_id = workouts::table
            .filter(workouts::user_id.eq(user_id))
            .filter(workouts::uuid.eq(workout_uuid))
            .select(workouts::id)
            .first::<i64>(&mut conn)
            .map_err(WorkoutExerciseError::from)?;
        let exercise_id = exercises::table
            .filter(exercises::user_id.eq(user_id))
            .filter(exercises::uuid.eq(exercise_uuid))
            .select(exercises::id)
            .first::<i64>(&mut conn)
            .map_err(WorkoutExerciseError::from)?;

        let workout_exercise = WorkoutExercise {
            workout_id,
            exercise_id,
            user_id,
            order,
        };

        diesel::insert_into(workout_exercises::table)
            .values(&workout_exercise)
            .execute(&mut conn)
            .map_err(WorkoutExerciseError::from)?;

        Ok(())
    }

    fn remove_exercise_from_workout(&self, user_id: i64, workout_uuid: Uuid, exercise_uuid: Uuid) -> Result<(), WorkoutExerciseError> {
        use crate::schema::public::{exercises, workouts, workout_exercises};
        let mut conn = db::config::establish_connection();

        let workout_id = workouts::table
            .filter(workouts::user_id.eq(user_id))
            .filter(workouts::uuid.eq(workout_uuid))
            .select(workouts::id)
            .first::<i64>(&mut conn)
            .map_err(WorkoutExerciseError::from)?;
        let exercise_id = exercises::table
            .filter(exercises::user_id.eq(user_id))
            .filter(exercises::uuid.eq(exercise_uuid))
            .select(exercises::id)
            .first::<i64>(&mut conn)
            .map_err(WorkoutExerciseError::from)?;

        let result = diesel::delete(workout_exercises::table)
            .filter(workout_exercises::user_id.eq(user_id))
            .filter(workout_exercises::workout_id.eq(workout_id))
            .filter(workout_exercises::exercise_id.eq(exercise_id))
            .execute(&mut conn)
            .map_err(WorkoutExerciseError::from)?;

        if result == 0 {
            return Err(WorkoutExerciseError::NotFound);
        }

        Ok(())
    }

    fn list_workout_exercises(&self, user_id: i64, workout_uuid: Uuid) -> Result<Vec<(Exercise, i32)>, WorkoutExerciseError> {
        use crate::schema::public::{exercises, workouts, workout_exercises};
        let mut conn = db::config::establish_connection();

        let workout_id = workouts::table
            .filter(workouts::user_id.eq(user_id))
            .filter(workouts::uuid.eq(workout_uuid))
            .select(workouts::id)
            .first::<i64>(&mut conn)
            .map_err(WorkoutExerciseError::from)?;

        exercises::table
            .inner_join(workout_exercises::table.on(
                exercises::id.eq(workout_exercises::exercise_id)
            ))
            .filter(workout_exercises::user_id.eq(user_id))
            .filter(workout_exercises::workout_id.eq(workout_id))
            .select((exercises::all_columns, workout_exercises::order))
            .load::<(Exercise, i32)>(&mut conn)
            .map_err(WorkoutExerciseError::from)
    }
} 
use diesel::prelude::*;
use uuid::Uuid;
use crate::{db, models::exercise::{Exercise, CreateExercise, UpdateExercise}};

#[derive(Debug)]
pub enum ExerciseError {
    NotFound,
    DatabaseError(diesel::result::Error),
    Unauthorized,
}

impl From<diesel::result::Error> for ExerciseError {
    fn from(err: diesel::result::Error) -> ExerciseError {
        match err {
            diesel::result::Error::NotFound => ExerciseError::NotFound,
            _ => ExerciseError::DatabaseError(err),
        }
    }
}

pub trait ExerciseRepository {
    fn create_exercise(&self, user_id: i64, exercise: CreateExercise) -> Result<Exercise, ExerciseError>;
    fn get_exercise(&self, user_id: i64, exercise_uuid: Uuid) -> Result<Exercise, ExerciseError>;
    fn list_exercises(&self, user_id: i64) -> Result<Vec<Exercise>, ExerciseError>;
    fn update_exercise(&self, user_id: i64, exercise_uuid: Uuid, exercise: UpdateExercise) -> Result<Exercise, ExerciseError>;
    fn delete_exercise(&self, user_id: i64, exercise_uuid: Uuid) -> Result<(), ExerciseError>;
}

pub struct PgExerciseRepository;

impl PgExerciseRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExerciseRepository for PgExerciseRepository {
    fn create_exercise(&self, user_id: i64, exercise: CreateExercise) -> Result<Exercise, ExerciseError> {
        use crate::schema::public::exercises;
        let mut conn = db::config::establish_connection();

        let new_exercise = Exercise::new(user_id, exercise.name, exercise.description);
        diesel::insert_into(exercises::table)
            .values(&new_exercise)
            .get_result(&mut conn)
            .map_err(ExerciseError::from)
    }

    fn get_exercise(&self, user_id: i64, exercise_uuid: Uuid) -> Result<Exercise, ExerciseError> {
        use crate::schema::public::exercises;
        let mut conn = db::config::establish_connection();

        exercises::table
            .filter(exercises::user_id.eq(user_id))
            .filter(exercises::uuid.eq(exercise_uuid))
            .first::<Exercise>(&mut conn)
            .map_err(ExerciseError::from)
    }

    fn list_exercises(&self, user_id: i64) -> Result<Vec<Exercise>, ExerciseError> {
        use crate::schema::public::exercises;
        let mut conn = db::config::establish_connection();

        exercises::table
            .filter(exercises::user_id.eq(user_id))
            .load::<Exercise>(&mut conn)
            .map_err(ExerciseError::from)
    }

    fn update_exercise(&self, user_id: i64, exercise_uuid: Uuid, exercise: UpdateExercise) -> Result<Exercise, ExerciseError> {
        use crate::schema::public::exercises;
        let mut conn = db::config::establish_connection();

        let exercise_exists = exercises::table
            .filter(exercises::user_id.eq(user_id))
            .filter(exercises::uuid.eq(exercise_uuid))
            .first::<Exercise>(&mut conn)
            .map_err(ExerciseError::from)?;

        diesel::update(exercises::table)
            .filter(exercises::uuid.eq(exercise_uuid))
            .set((
                exercises::name.eq(exercise.name.unwrap_or(exercise_exists.name)),
                exercises::description.eq(exercise.description.or(exercise_exists.description)),
                exercises::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .get_result(&mut conn)
            .map_err(ExerciseError::from)
    }

    fn delete_exercise(&self, user_id: i64, exercise_uuid: Uuid) -> Result<(), ExerciseError> {
        use crate::schema::public::exercises;
        let mut conn = db::config::establish_connection();

        let result = diesel::delete(exercises::table)
            .filter(exercises::user_id.eq(user_id))
            .filter(exercises::uuid.eq(exercise_uuid))
            .execute(&mut conn)
            .map_err(ExerciseError::from)?;

        if result == 0 {
            return Err(ExerciseError::NotFound);
        }

        Ok(())
    }
} 
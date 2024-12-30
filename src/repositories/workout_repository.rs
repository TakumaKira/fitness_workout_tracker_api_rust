use diesel::prelude::*;
use uuid::Uuid;
use crate::{db, models::workout::{Workout, CreateWorkout, UpdateWorkout}};

#[derive(Debug)]
pub enum WorkoutError {
    NotFound,
    DatabaseError(diesel::result::Error),
    Unauthorized,
}

impl From<diesel::result::Error> for WorkoutError {
    fn from(err: diesel::result::Error) -> WorkoutError {
        match err {
            diesel::result::Error::NotFound => WorkoutError::NotFound,
            _ => WorkoutError::DatabaseError(err),
        }
    }
}

pub trait WorkoutRepository {
    fn create_workout(&self, user_id: i64, workout: CreateWorkout) -> Result<Workout, WorkoutError>;
    fn get_workout(&self, user_id: i64, workout_uuid: Uuid) -> Result<Workout, WorkoutError>;
    fn list_workouts(&self, user_id: i64) -> Result<Vec<Workout>, WorkoutError>;
    fn update_workout(&self, user_id: i64, workout_uuid: Uuid, workout: UpdateWorkout) -> Result<Workout, WorkoutError>;
    fn delete_workout(&self, user_id: i64, workout_uuid: Uuid) -> Result<(), WorkoutError>;
}

pub struct PgWorkoutRepository;

impl PgWorkoutRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl WorkoutRepository for PgWorkoutRepository {
    fn create_workout(&self, user_id: i64, workout: CreateWorkout) -> Result<Workout, WorkoutError> {
        use crate::schema::public::workouts;
        let mut conn = db::config::establish_connection();

        let new_workout = Workout::new(
            user_id,
            workout.name,
            workout.description,
        );
        diesel::insert_into(workouts::table)
            .values(&new_workout)
            .get_result(&mut conn)
            .map_err(WorkoutError::from)
    }

    fn get_workout(&self, user_id: i64, workout_uuid: Uuid) -> Result<Workout, WorkoutError> {
        use crate::schema::public::workouts;
        let mut conn = db::config::establish_connection();

        let workout = workouts::table
            .filter(workouts::uuid.eq(workout_uuid))
            .first::<Workout>(&mut conn)
            .map_err(WorkoutError::from)?;

        if workout.user_id != user_id {
            return Err(WorkoutError::Unauthorized);
        }

        Ok(workout)
    }

    fn list_workouts(&self, user_id: i64) -> Result<Vec<Workout>, WorkoutError> {
        use crate::schema::public::workouts;
        let mut conn = db::config::establish_connection();

        workouts::table
            .filter(workouts::user_id.eq(user_id))
            .load::<Workout>(&mut conn)
            .map_err(WorkoutError::from)
    }

    fn update_workout(&self, user_id: i64, workout_uuid: Uuid, workout: UpdateWorkout) -> Result<Workout, WorkoutError> {
        use crate::schema::public::workouts;
        let mut conn = db::config::establish_connection();

        let workout_exists = workouts::table
            .filter(workouts::uuid.eq(workout_uuid))
            .filter(workouts::user_id.eq(user_id))
            .first::<Workout>(&mut conn)
            .map_err(|_| WorkoutError::Unauthorized)?;

        diesel::update(workouts::table)
            .filter(workouts::uuid.eq(workout_uuid))
            .set((
                workouts::name.eq(workout.name.unwrap_or(workout_exists.name)),
                workouts::description.eq(workout.description.or(workout_exists.description)),
            ))
            .get_result(&mut conn)
            .map_err(WorkoutError::from)
    }

    fn delete_workout(&self, user_id: i64, workout_uuid: Uuid) -> Result<(), WorkoutError> {
        use crate::schema::public::workouts;
        let mut conn = db::config::establish_connection();

        let result = diesel::delete(workouts::table)
            .filter(workouts::uuid.eq(workout_uuid))
            .filter(workouts::user_id.eq(user_id))
            .execute(&mut conn)
            .map_err(WorkoutError::from)?;

        if result == 0 {
            return Err(WorkoutError::NotFound);
        }

        Ok(())
    }
} 
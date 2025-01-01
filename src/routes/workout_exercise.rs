use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Resource, Responder};
use uuid::Uuid;

use crate::{
  models::workout_exercise::AddExerciseRequest,
  repositories::workout_exercise_repository::{WorkoutExerciseError, WorkoutExerciseRepository},
};

pub fn get_scope_workout_id_exercises_exercise_id<T: WorkoutExerciseRepository + 'static>() -> Resource {
  web::resource("/workouts/{workout_uuid}/exercises/{exercise_uuid}")
      .route(web::delete().to(remove_exercise_from_workout::<T>))
}

pub fn get_scope_workout_id_exercises<T: WorkoutExerciseRepository + 'static>() -> Resource {
  web::resource("/workouts/{workout_uuid}/exercises")
      .route(web::get().to(list_workout_exercises::<T>))
      .route(web::post().to(add_exercise_to_workout::<T>))
}

async fn add_exercise_to_workout<T: WorkoutExerciseRepository>(
  workout_uuid: web::Path<Uuid>,
  exercise: web::Json<AddExerciseRequest>,
  req: HttpRequest,
  repo: web::Data<T>,
) -> impl Responder {
  let user_id = *req.extensions().get::<i64>().unwrap();
  match repo.add_exercise_to_workout(user_id, *workout_uuid, exercise.exercise_uuid, exercise.order) {
    Ok(()) => HttpResponse::Ok().finish(),
    Err(e) => match e {
      WorkoutExerciseError::NotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::WorkoutNotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::ExerciseNotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::Unauthorized => HttpResponse::Forbidden().finish(),
      _ => HttpResponse::InternalServerError().finish(),
    }
  }
}

async fn list_workout_exercises<T: WorkoutExerciseRepository>(
  workout_uuid: web::Path<Uuid>,
  req: HttpRequest,
  repo: web::Data<T>,
) -> impl Responder {
  let user_id = *req.extensions().get::<i64>().unwrap();
  match repo.list_workout_exercises(user_id, *workout_uuid) {
    Ok(exercises) => HttpResponse::Ok().json(exercises),
    Err(e) => match e {
      WorkoutExerciseError::NotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::WorkoutNotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::ExerciseNotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::Unauthorized => HttpResponse::Forbidden().finish(),
      _ => HttpResponse::InternalServerError().finish(),
    }
  }
}

async fn remove_exercise_from_workout<T: WorkoutExerciseRepository>(
  path: web::Path<(Uuid, Uuid)>,
  req: HttpRequest,
  repo: web::Data<T>,
) -> impl Responder {
  let (workout_uuid, exercise_uuid) = path.into_inner();
  let user_id = *req.extensions().get::<i64>().unwrap();
  match repo.remove_exercise_from_workout(user_id, workout_uuid, exercise_uuid) {
    Ok(()) => HttpResponse::Ok().finish(),
    Err(e) => match e {
      WorkoutExerciseError::NotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::WorkoutNotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::ExerciseNotFound => HttpResponse::NotFound().finish(),
      WorkoutExerciseError::Unauthorized => HttpResponse::Forbidden().finish(),
      _ => HttpResponse::InternalServerError().finish(),
    }
  }
}
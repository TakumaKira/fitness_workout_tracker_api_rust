use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Resource, Responder, Scope};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{
    models::exercise::{CreateExercise, Exercise, UpdateExercise},
    repositories::exercise_repository::{ExerciseError, ExerciseRepository},
};

#[derive(Serialize, Deserialize)]
pub struct ExerciseResponse {
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl ExerciseResponse {
    fn from(exercise: &Exercise) -> Self {
        Self {
            uuid: exercise.uuid,
            name: exercise.name.clone(),
            description: exercise.description.clone(),
        }
    }
}

pub fn get_scope_exercise_id<T: ExerciseRepository + 'static>() -> Resource {
    web::resource("/exercises/{exercise_uuid}")
        .route(web::get().to(get_exercise::<T>))
        .route(web::put().to(update_exercise::<T>))
        .route(web::delete().to(delete_exercise::<T>))
}

pub fn get_scope<T: ExerciseRepository + 'static>() -> Scope {
    web::scope("/exercises")
        .route("", web::post().to(create_exercise::<T>))
        .route("", web::get().to(list_exercises::<T>))
}

async fn create_exercise<T: ExerciseRepository>(
    exercise: web::Json<CreateExercise>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();
    match repo.create_exercise(user_id, exercise.0) {
        Ok(exercise) => HttpResponse::Created().json(ExerciseResponse::from(&exercise)),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn list_exercises<T: ExerciseRepository>(
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();

    match repo.list_exercises(user_id) {
        Ok(exercises) => HttpResponse::Ok().json(
            exercises.iter().map(ExerciseResponse::from).collect::<Vec<_>>()
        ),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_exercise<T: ExerciseRepository>(
    exercise_uuid: web::Path<Uuid>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();

    match repo.get_exercise(user_id, *exercise_uuid) {
        Ok(exercise) => HttpResponse::Ok().json(ExerciseResponse::from(&exercise)),
        Err(ExerciseError::NotFound) => HttpResponse::NotFound().finish(),
        Err(ExerciseError::Unauthorized) => HttpResponse::Unauthorized().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn update_exercise<T: ExerciseRepository>(
    exercise_uuid: web::Path<Uuid>,
    exercise: web::Json<UpdateExercise>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();

    match repo.update_exercise(user_id, *exercise_uuid, exercise.0) {
        Ok(exercise) => HttpResponse::Ok().json(ExerciseResponse::from(&exercise)),
        Err(ExerciseError::NotFound) => HttpResponse::NotFound().finish(),
        Err(ExerciseError::Unauthorized) => HttpResponse::Unauthorized().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_exercise<T: ExerciseRepository>(
    exercise_uuid: web::Path<Uuid>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();

    match repo.delete_exercise(user_id, *exercise_uuid) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(ExerciseError::NotFound) => HttpResponse::NotFound().finish(),
        Err(ExerciseError::Unauthorized) => HttpResponse::Unauthorized().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
} 
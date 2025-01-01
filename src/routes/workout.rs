use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Resource, Responder, Scope};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    models::workout::{CreateWorkout, UpdateWorkout, Workout},
    repositories::workout_repository::{WorkoutError, WorkoutRepository},
};

#[derive(Serialize)]
struct WorkoutResponse {
    uuid: uuid::Uuid,
    name: String,
    description: Option<String>,
}

impl WorkoutResponse {
    fn from(workout: &Workout) -> Self {
        Self {
            uuid: workout.uuid,
            name: workout.name.clone(),
            description: workout.description.clone(),
        }
    }
}

pub fn get_scope_workout_id<T: WorkoutRepository + 'static>() -> Resource {
    web::resource("/workouts/{workout_uuid}")
        .route(web::get().to(get_workout::<T>))
        .route(web::put().to(update_workout::<T>))
        .route(web::delete().to(delete_workout::<T>))
}

pub fn get_scope<T: WorkoutRepository + 'static>() -> Scope {
    web::scope("/workouts")
        .route("", web::post().to(create_workout::<T>))
        .route("", web::get().to(list_workouts::<T>))
}

async fn create_workout<T: WorkoutRepository>(
    workout: web::Json<CreateWorkout>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();
    match repo.create_workout(user_id, workout.0) {
        Ok(workout) => HttpResponse::Created().json(WorkoutResponse::from(&workout)),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn list_workouts<T: WorkoutRepository>(
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();
    
    match repo.list_workouts(user_id) {
        Ok(workouts) => HttpResponse::Ok().json(
            workouts.iter().map(WorkoutResponse::from).collect::<Vec<_>>()
        ),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_workout<T: WorkoutRepository>(
    workout_uuid: web::Path<Uuid>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();
    match repo.get_workout(user_id, *workout_uuid) {
        Ok(workout) => HttpResponse::Ok().json(WorkoutResponse::from(&workout)),
        Err(WorkoutError::NotFound) => HttpResponse::NotFound().finish(),
        Err(WorkoutError::Unauthorized) => {
            HttpResponse::Unauthorized().finish()
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn update_workout<T: WorkoutRepository>(
    workout_uuid: web::Path<Uuid>,
    workout: web::Json<UpdateWorkout>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();
    
    match repo.update_workout(user_id, *workout_uuid, workout.0) {
        Ok(workout) => HttpResponse::Ok().json(WorkoutResponse::from(&workout)),
        Err(WorkoutError::NotFound) => HttpResponse::NotFound().finish(),
        Err(WorkoutError::Unauthorized) => {
            HttpResponse::Unauthorized().finish()
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_workout<T: WorkoutRepository>(
    workout_uuid: web::Path<Uuid>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i64>().unwrap();
    
    match repo.delete_workout(user_id, *workout_uuid) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(WorkoutError::NotFound) => HttpResponse::NotFound().finish(),
        Err(WorkoutError::Unauthorized) => {
            HttpResponse::Unauthorized().finish()
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


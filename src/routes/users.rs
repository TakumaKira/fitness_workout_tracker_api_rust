use actix_web::{web, post, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use crate::repositories::user_repository::UserRepository;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: uuid::Uuid,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub fn get_scope() -> Scope {
    web::scope("/users")
        .service(create_user)
}

#[post("")]
async fn create_user(
    user_data: web::Json<CreateUserRequest>
) -> impl Responder {
    let repo = UserRepository::new();
    
    match repo.create_user(user_data.email.clone(), user_data.password.clone()) {
        Ok(user) => {
            let response = UserResponse {
                id: user.id,
                email: user.email,
                created_at: chrono::DateTime::from_naive_utc_and_offset(
                    user.created_at,
                    chrono::Utc,
                ),
            };
            HttpResponse::Created().json(response)
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    }
} 
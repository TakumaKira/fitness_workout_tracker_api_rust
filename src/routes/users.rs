use actix_web::{web, post, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: String,
    email: String,
    created_at: chrono::DateTime<Utc>,
}

pub fn get_scope() -> Scope {
  web::scope("/users")
      .service(create_user)
}

#[post("")]
async fn create_user(user_data: web::Json<CreateUserRequest>) -> impl Responder {
    // TODO: Add actual user creation logic
    let user = UserResponse {
        id: Uuid::new_v4().to_string(),
        email: user_data.email.clone(),
        created_at: Utc::now(),
    };

    HttpResponse::Created().json(user)
} 
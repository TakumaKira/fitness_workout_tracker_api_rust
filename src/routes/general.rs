use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    content: String,
}

pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

pub async fn echo(msg: web::Json<Message>) -> impl Responder {
    HttpResponse::Ok().json(msg.0)
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
} 
use actix_web::{web, get, post, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    content: String,
}

pub fn get_scope() -> Scope {
    web::scope("")
        .service(hello)
        .service(echo)
        .service(health_check)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("It's running")
}

#[post("/echo")]
async fn echo(msg: web::Json<Message>) -> impl Responder {
    HttpResponse::Ok().json(msg.0)
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
} 
use actix_web::{get, web, HttpResponse, Responder, Scope};

pub fn get_scope() -> Scope {
    web::scope("")
        .service(get_workouts)
}

#[get("")]
async fn get_workouts() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}
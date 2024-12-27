use actix_web::{App, HttpServer, web};
use fitness_workout_tracker_api_rust::{
    routes,
    middleware::{csrf::CsrfProtection, session::SessionProtection}
};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    println!("Server starting at http://{}", address);
    
    HttpServer::new(move || {
        App::new()
            .wrap(CsrfProtection)
            .service(
                web::scope("/workouts")
                    .wrap(SessionProtection)
                    .service(routes::workout::get_scope())
            )
            .service(routes::auth::get_scope())
            .service(routes::general::get_scope())
    })
    .bind(&address)?
    .run()
    .await
}

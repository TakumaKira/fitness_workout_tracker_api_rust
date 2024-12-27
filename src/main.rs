use actix_web::{App, HttpServer};
use fitness_workout_tracker_api_rust::routes;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    println!("Server starting at http://{}", address);
    
    HttpServer::new(move || {
        App::new()
            .service(routes::auth::get_scope())
            .service(routes::users::get_scope())
            .service(routes::general::get_scope())
    })
    .bind(&address)?
    .run()
    .await
}

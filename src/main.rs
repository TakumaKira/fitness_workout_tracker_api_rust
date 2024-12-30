use actix_web::{App, HttpServer, web};
use fitness_workout_tracker_api_rust::{
    middleware::{csrf::CsrfProtection, session::SessionProtection}, repositories::{auth_repository::PgAuthRepository, workout_repository::PgWorkoutRepository}, routes
};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    let auth_repo = web::Data::new(PgAuthRepository::new());
    let workout_repo = web::Data::new(PgWorkoutRepository::new());

    println!("Server starting at http://{}", address);
    
    HttpServer::new(move || {
        App::new()
            .wrap(CsrfProtection::<PgAuthRepository>::new())
            .app_data(auth_repo.clone())
            .service(routes::auth::get_scope::<PgAuthRepository>())
            .service(
                web::scope("")
                    .wrap(SessionProtection::<PgAuthRepository>::new())
                    .app_data(workout_repo.clone())
                    .service(routes::workout::get_scope_with_resource::<PgWorkoutRepository>())
                    .service(routes::workout::get_scope::<PgWorkoutRepository>())
            )
            .service(routes::general::get_scope())
    })
    .bind(&address)?
    .run()
    .await
}

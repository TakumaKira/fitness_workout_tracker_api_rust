use actix_web::{App, HttpServer, web};
use fitness_workout_tracker_api_rust::{
    middleware::csrf::CsrfProtection, repositories::auth_repository::PgAuthRepository, routes
};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    let auth_repo = web::Data::new(PgAuthRepository::new());

    println!("Server starting at http://{}", address);
    
    HttpServer::new(move || {
        App::new()
            .wrap(CsrfProtection::<PgAuthRepository>::new())
            .app_data(auth_repo.clone())
            .service(routes::auth::get_scope::<PgAuthRepository>())
            .service(routes::general::get_scope())
    })
    .bind(&address)?
    .run()
    .await
}

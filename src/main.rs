mod routes;
mod db;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize database pool
    let pool = db::config::create_pool()
        .await
        .expect("Failed to create database pool");

    println!("Server starting at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::general::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

mod routes;

use actix_web::{web, App, HttpServer};
use routes::general;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting at http://127.0.0.1:8080");
    
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(general::hello))
            .route("/echo", web::post().to(general::echo))
            .route("/health", web::get().to(general::health_check))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

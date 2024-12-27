mod routes;
mod db;

use actix_web::{web, App, HttpServer};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    println!("Server starting at http://{}", address);
    
    let conn = db::config::create_connection();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conn.clone()))
            .configure(routes::general::configure)
    })
    .bind(&address)?
    .run()
    .await
}

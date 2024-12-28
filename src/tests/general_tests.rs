use actix_web::{test, App};
use serde_json::json;
use crate::routes::general;

#[actix_web::test]
async fn test_hello() {
    let app = test::init_service(
        App::new()
            .service(general::get_scope())
    ).await;

    let req = test::TestRequest::get()
        .uri("/")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    assert_eq!(body, "It's running");
}

#[actix_web::test]
async fn test_echo() {
    let app = test::init_service(
        App::new()
            .service(general::get_scope())
    ).await;

    let message = "Hello, test!";
    let req = test::TestRequest::post()
        .uri("/echo")
        .set_json(json!({ "content": message }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["content"], message);
}

#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(
        App::new()
            .service(general::get_scope())
    ).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("status").is_some());
    assert_eq!(body["status"], "healthy");
    assert!(body.get("timestamp").is_some());
} 
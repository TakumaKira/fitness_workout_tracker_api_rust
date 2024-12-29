use actix_web::{cookie::Cookie, test, web, App, HttpResponse};
use serde_json::json;
use std::sync::Mutex;
use crate::{
    middleware::{csrf::CsrfProtection, session::SessionProtection}, models::{session::Session, temp_session::TempSession, user::User}, repositories::auth_repository::{AuthError, AuthRepository}, routes::auth
};

pub struct MockAuthRepo {
    users: Mutex<Vec<User>>,
    sessions: Mutex<Vec<Session>>,
    temp_sessions: Mutex<Vec<TempSession>>,
}

impl MockAuthRepo {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(vec![]),
            sessions: Mutex::new(vec![]),
            temp_sessions: Mutex::new(vec![]),
        }
    }
}

impl AuthRepository for MockAuthRepo {
    fn create_temp_session(&self, csrf_token: String) -> Result<TempSession, AuthError> {
        let new_session = TempSession::new(uuid::Uuid::new_v4().to_string(), csrf_token);
        let session = TempSession {
            id: 1,  // Mock ID
            session_id: new_session.session_id,
            csrf_token: new_session.csrf_token,
            created_at: new_session.created_at,
            expires_at: new_session.expires_at,
        };
        self.temp_sessions.lock().unwrap().push(session.clone());
        Ok(session)
    }

    fn create_session(&self, user_id: i64, session_id: String, csrf_token: String) -> Result<Session, AuthError> {
        let new_session = Session::new(user_id, session_id, csrf_token);
        let session = Session {
            id: 1,  // Mock ID
            user_id: new_session.user_id,
            token: new_session.token,
            csrf_token: new_session.csrf_token,
            expires_at: new_session.expires_at,
            created_at: new_session.created_at,
        };
        self.sessions.lock().unwrap().push(session.clone());
        Ok(session)
    } 

    fn validate_csrf(&self, session_id: &str, csrf_token: &str) -> Result<(), AuthError> {
        let temp_sessions = self.temp_sessions.lock().unwrap();
        temp_sessions.iter()
            .find(|s| s.session_id == session_id && s.csrf_token == csrf_token)
            .ok_or(AuthError::InvalidSession)?;
        Ok(())
    }

    fn verify_credentials(&self, email: String, password: String) -> Result<User, AuthError> {
        let users = self.users.lock().unwrap();
        users.iter()
            .find(|u| u.email == email && u.password_hash == password)
            .cloned()
            .ok_or(AuthError::InvalidCredentials)
    }

    fn create_user(&self, email: String, password: String) -> Result<User, AuthError> {
        let mut users = self.users.lock().unwrap();
        if users.iter().any(|u| u.email == email) {
            return Err(AuthError::DuplicateEmail);
        }
        let new_user = User::new(email.clone(), password);
        let user = User {
            id: 1,  // Mock ID
            uuid: new_user.uuid,
            email,
            password_hash: new_user.password_hash,
            created_at: new_user.created_at,
            updated_at: new_user.updated_at,
        };
        users.push(user.clone());
        Ok(user)
    }

    fn validate_session(&self, session_token: &str) -> Result<(), AuthError> {
        let sessions = self.sessions.lock().unwrap();
        sessions.iter()
            .find(|s| s.token == session_token)
            .ok_or(AuthError::InvalidSession)?;
        Ok(())
    }

    fn invalidate_session(&self, session_token: &str) -> Result<(), AuthError> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|s| s.token != session_token);
        Ok(())
    }
}

#[actix_web::test]
async fn test_csrf_token_flow() {
    let mock_repo = web::Data::new(MockAuthRepo::new());
    
    let app = test::init_service(
        App::new()
            .wrap(CsrfProtection::<MockAuthRepo>::new())
            .app_data(mock_repo.clone())
            .service(auth::get_scope::<MockAuthRepo>())
    ).await;

    // Get CSRF token
    let req = test::TestRequest::get()
        .uri("/auth/csrf-token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let session_cookie = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");
    let session_id = session_cookie.value();
    let next_cookie = Cookie::new("session_id", session_id.to_string());

    let body: serde_json::Value = test::read_body_json(resp).await;
    let csrf_token = body["csrf_token"].as_str().unwrap();
    
    // Try register with CSRF token
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .cookie(next_cookie.clone())
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Try register without CSRF token (should fail)
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .cookie(next_cookie.clone())
        .set_json(json!({
            "email": "test2@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_login_flow() {
    let mock_repo = web::Data::new(MockAuthRepo::new());
    
    let app = test::init_service(
        App::new()
            .app_data(mock_repo.clone())
            .service(auth::get_scope::<MockAuthRepo>())
    ).await;

    // First get CSRF token
    let req = test::TestRequest::get()
        .uri("/auth/csrf-token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let session_cookie = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");
    let session_id = session_cookie.value();
    let next_cookie = Cookie::new("session_id", session_id.to_string());

    let body: serde_json::Value = test::read_body_json(resp).await;
    let csrf_token = body["csrf_token"].as_str().unwrap();

    // Register a user first
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .cookie(next_cookie.clone())
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();
    let _ = test::call_service(&app, req).await;

    // Try login with correct credentials
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .cookie(next_cookie.clone())
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Try login with wrong password
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .cookie(next_cookie.clone())
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "wrongpassword"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Try login with non-existent email
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .cookie(next_cookie)
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "nonexistent@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
} 

#[actix_web::test]
async fn test_guarded_route_on_register() {
    let mock_repo = web::Data::new(MockAuthRepo::new());
    
    let app = test::init_service(
        App::new()
            .app_data(mock_repo.clone())
            .service(auth::get_scope::<MockAuthRepo>())
            .service(
                web::scope("/api")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() }))
            )
    ).await;

    // Try accessing protected route without session
    let req = test::TestRequest::get()
        .uri("/api/test")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Get session by logging in
    let req = test::TestRequest::get()
        .uri("/auth/csrf-token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let session_cookie = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");
    let session_id = session_cookie.value();
    let next_cookie = Cookie::new("session_id", session_id.to_string());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let csrf_token = body["csrf_token"].as_str().unwrap();

    // Register and login
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .cookie(next_cookie)
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let session_cookie = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");
    let session_id = session_cookie.value();
    let next_cookie = Cookie::new("session_id", session_id.to_string());

    // Try accessing protected route with valid session
    let req = test::TestRequest::get()
        .uri("/api/test")
        .cookie(next_cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Try accessing protected route with invalid session
    let req = test::TestRequest::get()
        .uri("/api/test")
        .cookie(Cookie::new("session_id", "invalid-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_guarded_route_on_login() {
    let mock_repo = web::Data::new(MockAuthRepo::new());
    
    let app = test::init_service(
        App::new()
            .app_data(mock_repo.clone())
            .service(auth::get_scope::<MockAuthRepo>())
            .service(
                web::scope("/api")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() }))
            )
    ).await;

    // First register a user
    let req = test::TestRequest::get()
        .uri("/auth/csrf-token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let session_cookie = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");
    let session_id = session_cookie.value();
    let next_cookie = Cookie::new("session_id", session_id.to_string());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let csrf_token = body["csrf_token"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .cookie(next_cookie)
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();
    let _ = test::call_service(&app, req).await;

    // Get new CSRF token for login
    let req = test::TestRequest::get()
        .uri("/auth/csrf-token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let session_cookie = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");
    let session_id = session_cookie.value();
    let next_cookie = Cookie::new("session_id", session_id.to_string());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let csrf_token = body["csrf_token"].as_str().unwrap();

    // Login
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .cookie(next_cookie)
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let session = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");

    // Try accessing protected route with valid session from login
    let req = test::TestRequest::get()
        .uri("/api/test")
        .cookie(session.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Try accessing protected route with invalid session
    let req = test::TestRequest::get()
        .uri("/api/test")
        .cookie(Cookie::new("session_id", "invalid-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_logout() {
    let mock_repo = web::Data::new(MockAuthRepo::new());
    
    let app = test::init_service(
        App::new()
            .app_data(mock_repo.clone())
            .service(auth::get_scope::<MockAuthRepo>())
            .service(
                web::scope("/api")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() }))
            )
    ).await;

    // Get CSRF token and register
    let req = test::TestRequest::get()
        .uri("/auth/csrf-token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let session_cookie = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");
    let session_id = session_cookie.value();
    let next_cookie = Cookie::new("session_id", session_id.to_string());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let csrf_token = body["csrf_token"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .cookie(next_cookie)
        .insert_header(("x-csrf-token", csrf_token))
        .set_json(json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let session = resp.response().cookies()
        .find(|c| c.name() == "session_id")
        .expect("Session cookie not found");

    // Verify session works
    let req = test::TestRequest::get()
        .uri("/api/test")
        .cookie(session.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Logout
    let req = test::TestRequest::post()
        .uri("/auth/logout")
        .cookie(session.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify session is invalidated
    let req = test::TestRequest::get()
        .uri("/api/test")
        .cookie(session.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}
use actix_web::{cookie::{Cookie, SameSite}, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use csrf::CsrfToken;
use crate::{models::user::User, repositories::auth_repository::{AuthError, AuthRepository}};
use time::Duration;

#[derive(Serialize)]
pub struct TokenResponse {
    csrf_token: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    uuid: uuid::Uuid,
    email: String,
}

impl LoginResponse {
    fn new(user: &User) -> Self {
        Self {
            uuid: user.uuid,
            email: user.email.clone(),
        }
    }
}

fn create_auth_response(
    user: User, 
    session_id: String,
    status: actix_web::http::StatusCode,
) -> Result<HttpResponse, AuthError> {
    let response = LoginResponse::new(&user);

    Ok(HttpResponse::build(status)
        .cookie(
            Cookie::build("session_id", session_id)
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .finish()
        )
        .json(response))
}

pub fn get_scope<T: AuthRepository + 'static>() -> Scope {
    web::scope("/auth")
        .route("/csrf-token", web::get().to(get_csrf_token::<T>))
        .route("/register", web::post().to(register::<T>))
        .route("/login", web::post().to(login::<T>))
        .route("/logout", web::post().to(logout::<T>))
}

async fn get_csrf_token<T: AuthRepository>(repo: web::Data<T>) -> impl Responder {
    let random_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    let token = CsrfToken::new(random_bytes);
    let temp_session = repo.create_temp_session(token.b64_string()).unwrap();
    HttpResponse::Ok()
        .cookie(
            Cookie::build("session_id", temp_session.session_id)
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .finish()
        )
        .json(TokenResponse {
            csrf_token: temp_session.csrf_token
        })
}

async fn register<T: AuthRepository>(
    user_data: web::Json<RegisterRequest>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    match repo.create_user(user_data.email.clone(), user_data.password.clone()) {
        Ok(user) => {
            let csrf_token = req.headers()
                .get("x-csrf-token")
                .and_then(|h| h.to_str().ok())
                .unwrap()
                .to_string();
            let session_id = req.cookie("session_id").unwrap().value().to_string();
            repo.create_session(user.id, session_id.clone(), csrf_token).unwrap();
            create_auth_response(user, session_id, StatusCode::CREATED)
                .unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
        },
        Err(AuthError::DuplicateEmail) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "Email already exists"
            }))
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

async fn login<T: AuthRepository>(
    user_data: web::Json<LoginRequest>,
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    match repo.verify_credentials(user_data.email.clone(), user_data.password.clone()) {
        Ok(user) => {
            let csrf_token = req.headers()
                .get("x-csrf-token")
                .and_then(|h| h.to_str().ok())
                .unwrap()
                .to_string();
            let session_id = req.cookie("session_id").unwrap().value().to_string();
            repo.create_session(user.id, session_id.clone(), csrf_token).unwrap();
            create_auth_response(user, session_id.clone(), StatusCode::OK)
                .unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
        },
        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }))
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

async fn logout<T: AuthRepository>(
    req: HttpRequest,
    repo: web::Data<T>,
) -> impl Responder {
    if let Some(cookie) = req.cookie("session_id") {
        let _ = repo.invalidate_session(cookie.value());  // Best effort deletion

        HttpResponse::Ok()
            .cookie(
                Cookie::build("session", "")
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Strict)
                    .max_age(Duration::seconds(0))
                    .finish()
            )
            .finish()
    } else {
        HttpResponse::Ok().finish()
    }
}

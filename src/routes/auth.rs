use actix_web::{cookie::{Cookie, SameSite}, get, post, web, HttpResponse, Responder, Scope, HttpRequest};
use serde::{Deserialize, Serialize};
use csrf::CsrfToken;
use crate::repositories::user_repository::{UserRepository, UserError};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    csrf_token: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    uuid: uuid::Uuid,
    email: String,
}

pub fn get_scope() -> Scope {
    web::scope("/auth")
        .service(get_csrf_token)
        .service(login)
}

#[get("/csrf-token")]
async fn get_csrf_token() -> impl Responder {
    let random_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    let token = CsrfToken::new(random_bytes);
    
    let repo = UserRepository::new();
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

#[post("/login")]
async fn login(
    user_data: web::Json<LoginRequest>,
    req: HttpRequest,
) -> impl Responder {
    let repo = UserRepository::new();

    // Validate session_id and CSRF token pair
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing session cookie"
        }))
    };

    let csrf_token = match req.headers().get("x-csrf-token") {
        Some(header) => header.to_str().unwrap_or_default().to_string(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing CSRF token"
        }))
    };

    if let Err(_) = repo.validate_csrf(&session_id, &csrf_token) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid CSRF token"
        }));
    }

    match repo.verify_credentials(user_data.email.clone(), user_data.password.clone()) {
        Ok(user) => {
            let session = repo.create_session(user.id, csrf_token).unwrap();
            
            let response = LoginResponse {
                uuid: user.uuid,
                email: user.email,
            };

            HttpResponse::Ok()
                .cookie(
                    Cookie::build("session", session.token)
                        .http_only(true)
                        .secure(true)
                        .same_site(SameSite::Strict)
                        .finish()
                )
                .json(response)
        },
        Err(UserError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }))
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    }
} 
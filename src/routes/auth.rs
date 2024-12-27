use actix_web::{cookie::{Cookie, SameSite}, get, http::StatusCode, post, web, HttpRequest, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use csrf::CsrfToken;
use crate::{models::user::User, repositories::auth_repository::{AuthError, AuthRepository}};

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
    csrf_token: String,
    repo: &AuthRepository,
    status: actix_web::http::StatusCode,
) -> Result<HttpResponse, AuthError> {
    let session = repo.create_session(user.id, csrf_token)?;
    let response = LoginResponse::new(&user);

    Ok(HttpResponse::build(status)
        .cookie(
            Cookie::build("session", session.token)
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .finish()
        )
        .json(response))
}

pub fn get_scope() -> Scope {
    web::scope("/auth")
        .service(get_csrf_token)
        .service(register)
        .service(login)
}

#[get("/csrf-token")]
async fn get_csrf_token() -> impl Responder {
    let random_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    let token = CsrfToken::new(random_bytes);
    
    let repo = AuthRepository::new();
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

#[post("/register")]
async fn register(
    user_data: web::Json<RegisterRequest>,
    req: HttpRequest,
) -> impl Responder {
    let repo = AuthRepository::new();
    
    match repo.create_user(user_data.email.clone(), user_data.password.clone()) {
        Ok(user) => {
            let csrf_token = req.headers()
                .get("x-csrf-token")
                .and_then(|h| h.to_str().ok())
                .unwrap()
                .to_string();

            create_auth_response(user, csrf_token, &repo, StatusCode::CREATED)
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

#[post("/login")]
async fn login(
    user_data: web::Json<LoginRequest>,
    req: HttpRequest,
) -> impl Responder {
    let repo = AuthRepository::new();
    
    match repo.verify_credentials(user_data.email.clone(), user_data.password.clone()) {
        Ok(user) => {
            let csrf_token = req.headers()
                .get("x-csrf-token")
                .and_then(|h| h.to_str().ok())
                .unwrap()
                .to_string();

            create_auth_response(user, csrf_token, &repo, StatusCode::OK)
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

use diesel::prelude::*;
use crate::{db, models::user::User, models::session::Session, models::temp_session::TempSession};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString, PasswordHash},
    Argon2, PasswordHasher, PasswordVerifier,
};

#[derive(Debug)]
pub enum UserError {
    DuplicateEmail,
    DatabaseError(diesel::result::Error),
    InvalidCredentials,
    InvalidSession,
    InvalidCsrf,
}

impl From<diesel::result::Error> for UserError {
    fn from(err: diesel::result::Error) -> UserError {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => UserError::DuplicateEmail,
            _ => UserError::DatabaseError(err),
        }
    }
}

pub struct UserRepository;

impl UserRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_user(&self, email: String, password: String) -> Result<User, UserError> {
        use crate::schema::public::users;
        let mut conn = db::config::establish_connection();

        // Wrap everything in a transaction
        conn.transaction(|conn| {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            let new_user = User::new(email, password_hash);

            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(conn)
                .map_err(UserError::from)?;

            users::table
                .filter(users::uuid.eq(new_user.uuid))
                .first(conn)
                .map_err(UserError::from)
        })
    }

    pub fn verify_credentials(&self, email: String, password: String) -> Result<User, UserError> {
        use crate::schema::public::users;
        let mut conn = db::config::establish_connection();

        let user = users::table
            .filter(users::email.eq(email))
            .first::<User>(&mut conn)
            .map_err(|_| UserError::InvalidCredentials)?;

        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| UserError::InvalidCredentials)?;

        if argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok() {
            Ok(user)
        } else {
            Err(UserError::InvalidCredentials)
        }
    }

    pub fn create_session(&self, user_id: i64, csrf_token: String) -> Result<Session, UserError> {
        use crate::schema::public::sessions;
        let mut conn = db::config::establish_connection();

        let new_session = Session::new(user_id, csrf_token);
        let token = new_session.token.clone(); // Clone token before insert

        diesel::insert_into(sessions::table)
            .values(&new_session)
            .execute(&mut conn)
            .map_err(UserError::from)?;

        sessions::table
            .filter(sessions::token.eq(token))
            .first(&mut conn)
            .map_err(UserError::from)
    }

    pub fn create_temp_session(&self, csrf_token: String) -> Result<TempSession, UserError> {
        use crate::schema::public::temp_sessions;
        let mut conn = db::config::establish_connection();

        let session_id = uuid::Uuid::new_v4().to_string();
        let new_temp_session = TempSession::new(session_id.clone(), csrf_token);

        diesel::insert_into(temp_sessions::table)
            .values(&new_temp_session)
            .execute(&mut conn)
            .map_err(UserError::from)?;

        temp_sessions::table
            .filter(temp_sessions::session_id.eq(session_id))
            .first(&mut conn)
            .map_err(UserError::from)
    }

    pub fn validate_csrf(&self, session_id: &str, csrf_token: &str) -> Result<(), UserError> {
        use crate::schema::public::temp_sessions;
        let mut conn = db::config::establish_connection();

        let now = chrono::Utc::now().naive_utc();
        
        temp_sessions::table
            .filter(temp_sessions::dsl::session_id.eq(session_id))
            .filter(temp_sessions::dsl::expires_at.gt(now))
            .filter(temp_sessions::dsl::csrf_token.eq(csrf_token))
            .first::<TempSession>(&mut conn)
            .map_err(|_| UserError::InvalidSession)?;

        Ok(())
    }
} 
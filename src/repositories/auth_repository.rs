use diesel::prelude::*;
use crate::{db, models::user::User, models::session::Session, models::temp_session::TempSession};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString, PasswordHash},
    Argon2, PasswordHasher, PasswordVerifier,
};

#[derive(Debug)]
pub enum AuthError {
    DuplicateEmail,
    DatabaseError(diesel::result::Error),
    InvalidCredentials,
    InvalidSession,
    InvalidCsrf,
    NotFound,
}

impl From<diesel::result::Error> for AuthError {
    fn from(err: diesel::result::Error) -> AuthError {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => AuthError::DuplicateEmail,
            _ => AuthError::DatabaseError(err),
        }
    }
}

pub trait AuthRepository {
    fn create_temp_session(&self, csrf_token: String) -> Result<TempSession, AuthError>;
    fn create_session(&self, user_id: i64, session_id: String, csrf_token: String) -> Result<Session, AuthError>;
    fn validate_csrf(&self, session_id: &str, csrf_token: &str) -> Result<(), AuthError>;
    fn verify_credentials(&self, email: String, password: String) -> Result<User, AuthError>;
    fn create_user(&self, email: String, password: String) -> Result<User, AuthError>;
    fn validate_session(&self, session_token: &str) -> Result<i64, AuthError>;
    fn invalidate_session(&self, session_token: &str) -> Result<(), AuthError>;
    fn delete_user(&self, session_token: &str) -> Result<(), AuthError>;
}

pub struct PgAuthRepository;

impl PgAuthRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl AuthRepository for PgAuthRepository {
    fn create_temp_session(&self, csrf_token: String) -> Result<TempSession, AuthError> {
        use crate::schema::public::temp_sessions;
        let mut conn = db::config::establish_connection();

        let session_id = uuid::Uuid::new_v4().to_string();
        let new_temp_session = TempSession::new(session_id.clone(), csrf_token);

        diesel::insert_into(temp_sessions::table)
            .values(&new_temp_session)
            .execute(&mut conn)
            .map_err(AuthError::from)?;

        temp_sessions::table
            .filter(temp_sessions::session_id.eq(session_id))
            .first(&mut conn)
            .map_err(AuthError::from)
    }

    fn create_session(&self, user_id: i64, session_id: String, csrf_token: String) -> Result<Session, AuthError> {
        use crate::schema::public::sessions;
        let mut conn = db::config::establish_connection();

        let new_session = Session::new(user_id, session_id, csrf_token);
        let token = new_session.token.clone(); // Clone token before insert

        diesel::insert_into(sessions::table)
            .values(&new_session)
            .execute(&mut conn)
            .map_err(AuthError::from)?;

        sessions::table
            .filter(sessions::token.eq(token))
            .first(&mut conn)
            .map_err(AuthError::from)
    }

    fn validate_csrf(&self, session_id: &str, csrf_token: &str) -> Result<(), AuthError> {
        use crate::schema::public::temp_sessions;
        let mut conn = db::config::establish_connection();
        let now = chrono::Utc::now().naive_utc();

        // Delete expired sessions first
        diesel::delete(temp_sessions::table)
            .filter(temp_sessions::expires_at.lt(now))
            .execute(&mut conn)
            .map_err(AuthError::from)?;
        
        // Then validate the current session
        temp_sessions::table
            .filter(temp_sessions::dsl::session_id.eq(session_id))
            .filter(temp_sessions::dsl::expires_at.gt(now))
            .filter(temp_sessions::dsl::csrf_token.eq(csrf_token))
            .first::<TempSession>(&mut conn)
            .map_err(|_| AuthError::InvalidSession)?;

        Ok(())
    }

    fn verify_credentials(&self, email: String, password: String) -> Result<User, AuthError> {
        use crate::schema::public::users;
        let mut conn = db::config::establish_connection();

        let user = users::table
            .filter(users::email.eq(email))
            .first::<User>(&mut conn)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        if argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok() {
            Ok(user)
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    fn validate_session(&self, session_token: &str) -> Result<i64, AuthError> {
        use crate::schema::public::sessions;
        let mut conn = db::config::establish_connection();
        let now = chrono::Utc::now().naive_utc();

        // Delete expired sessions first
        diesel::delete(sessions::table)
            .filter(sessions::expires_at.lt(now))
            .execute(&mut conn)
            .map_err(AuthError::from)?;

        // Validate session and return user_id
        let session = sessions::table
            .filter(sessions::token.eq(session_token))
            .filter(sessions::expires_at.gt(now))
            .first::<Session>(&mut conn)
            .map_err(|_| AuthError::InvalidSession)?;

        Ok(session.user_id)
    }

    fn invalidate_session(&self, session_token: &str) -> Result<(), AuthError> {
        use crate::schema::public::sessions;
        let mut conn = db::config::establish_connection();
        diesel::delete(sessions::table)
            .filter(sessions::token.eq(session_token))
            .execute(&mut conn)
            .map_err(AuthError::from)?;

        Ok(())
    }

    fn create_user(&self, email: String, password: String) -> Result<User, AuthError> {
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
                .map_err(AuthError::from)?;

            users::table
                .filter(users::uuid.eq(new_user.uuid))
                .first(conn)
                .map_err(AuthError::from)
        })
    }

    fn delete_user(&self, session_token: &str) -> Result<(), AuthError> {
        use crate::schema::public::{users, sessions};
        let mut conn = db::config::establish_connection();

        // First validate session
        let session = sessions::table
            .filter(sessions::token.eq(session_token))
            .first::<Session>(&mut conn)
            .map_err(|_| AuthError::InvalidSession)?;

        // Delete user associated with session
        diesel::delete(users::table)
            .filter(users::id.eq(session.user_id))
            .execute(&mut conn)
            .map_err(AuthError::from)?;

        Ok(())
    }
} 
use diesel::prelude::*;
use crate::{db, models::user::User};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

#[derive(Debug)]
pub enum UserError {
    DuplicateEmail,
    DatabaseError(diesel::result::Error),
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
} 
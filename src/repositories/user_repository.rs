use diesel::prelude::*;
use crate::{db, models::user::User};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

pub struct UserRepository;

impl UserRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_user(&self, email: String, password: String) -> QueryResult<User> {
        use crate::schema::public::users;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let new_user = User::new(email, password_hash);

        let mut conn = db::config::establish_connection();

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut conn)
            .expect("Error creating user");

        let user = users::table
            .filter(users::uuid.eq(new_user.uuid))
            .first(&mut conn)
            .unwrap();

        Ok(user)
    }
} 
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    PasswordHasher,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use http::StatusCode;
use thiserror::Error;

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(OsRng);
    let argon2 = argon2::Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("Incorrect username or password")]
    Unauthorized,
    #[error("User not found")]
    NotFound,
    #[error("Internal server error")]
    InternalServerError,
}

impl From<VerifyError> for StatusCode {
    fn from(value: VerifyError) -> Self {
        match value {
            VerifyError::Unauthorized => StatusCode::UNAUTHORIZED,
            VerifyError::NotFound => StatusCode::NOT_FOUND,
            VerifyError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    PasswordHasher,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use color_eyre::Result as AnyResult;
use uuid::Uuid;

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

pub fn decode_uuid_list_param(encoded: &Option<String>) -> AnyResult<Option<Vec<Uuid>>> {
    match encoded {
        Some(list) => {
            let ingredient_ids = urlencoding::decode(list)?;
            let ingredient_ids = serde_json::from_str::<Vec<Uuid>>(&ingredient_ids)?;
            Ok(Some(ingredient_ids))
        }
        None => Ok(None),
    }
}

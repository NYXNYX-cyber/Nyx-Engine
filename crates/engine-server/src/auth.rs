use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const JWT_SECRET: &[u8] = b"nyx-custom-engine-secret-key-2026";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(password_hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

pub fn create_jwt(user_id: &str, username: &str) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: now + (30 * 24 * 3600), // 30 days
    };

    let key = EncodingKey::from_secret(JWT_SECRET).map_err(|e| e.to_string())?;

    encode(&Header::default(), &claims, &key)
        .map_err(|e| e.to_string())
}

pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let key = DecodingKey::from_secret(JWT_SECRET).map_err(|e| e.to_string())?;

    decode::<Claims>(
        token,
        &key,
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| e.to_string())
}

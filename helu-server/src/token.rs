use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Claims {
    pub sub: String,
    pub method: String,
    pub exp: usize,
}

#[allow(dead_code)]
pub struct TokenManager {
    secret: String,
}

#[allow(dead_code)]
impl TokenManager {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn issue_token(&self, username: &str, method: &str, ttl_secs: u64) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let claims = Claims {
            sub: username.to_string(),
            method: method.to_string(),
            exp: (now + ttl_secs) as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes())
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default()
        )?;

        Ok(token_data.claims)
    }
}

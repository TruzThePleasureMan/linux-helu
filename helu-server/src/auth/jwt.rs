use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fs;
use anyhow::{Context, Result};
use crate::config::JwtConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct HeluClaims {
    pub sub: String,        // username
    pub iat: i64,           // issued at
    pub exp: i64,           // expires (configurable, default 1 hour)
    pub method: String,     // auth method used
    pub jti: String,        // unique token ID (UUID)
    pub iss: String,        // "helu-server"
}

pub struct TokenManager {
    algorithm: Algorithm,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    ttl_secs: u64,
}

impl TokenManager {
    pub fn new(config: &JwtConfig) -> Result<Self> {
        let (algorithm, encoding_key, decoding_key) = if config.algorithm == "RS256" {
            let priv_path = config.private_key_path.as_ref().context("Missing RS256 private key path")?;
            let pub_path = config.public_key_path.as_ref().context("Missing RS256 public key path")?;

            let priv_key_pem = fs::read(priv_path).context(format!("Failed to read RS256 private key at {}", priv_path))?;
            let pub_key_pem = fs::read(pub_path).context(format!("Failed to read RS256 public key at {}", pub_path))?;

            let enc_key = EncodingKey::from_rsa_pem(&priv_key_pem).context("Invalid RS256 private key PEM")?;
            let dec_key = DecodingKey::from_rsa_pem(&pub_key_pem).context("Invalid RS256 public key PEM")?;

            (Algorithm::RS256, enc_key, dec_key)
        } else {
            // HS256 default
            let secret = config.secret.as_ref().context("Missing HS256 secret")?;
            let enc_key = EncodingKey::from_secret(secret.as_bytes());
            let dec_key = DecodingKey::from_secret(secret.as_bytes());

            (Algorithm::HS256, enc_key, dec_key)
        };

        Ok(Self {
            algorithm,
            encoding_key,
            decoding_key,
            ttl_secs: config.ttl_secs,
        })
    }

    pub fn issue_token(&self, username: &str, method: &str) -> Result<String> {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let claims = HeluClaims {
            sub: username.to_string(),
            iat: now,
            exp: now + self.ttl_secs as i64,
            method: method.to_string(),
            jti: Uuid::new_v4().to_string(),
            iss: "helu-server".to_string(),
        };

        let mut header = Header::new(self.algorithm);
        header.kid = Some("helu-key-1".to_string()); // Optional, for future key rotation

        encode(&header, &claims, &self.encoding_key).context("Failed to encode JWT")
    }

    pub fn verify_token(&self, token: &str) -> Result<HeluClaims> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&["helu-server"]);

        let token_data = decode::<HeluClaims>(
            token,
            &self.decoding_key,
            &validation
        ).context("Failed to decode JWT")?;

        Ok(token_data.claims)
    }
}

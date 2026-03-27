use sqlx::PgPool;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use anyhow::Result;

pub async fn verify_api_key(pool: &PgPool, key: &str) -> Result<Option<uuid::Uuid>> {
    let mut parts = key.splitn(2, '_');
    if parts.next() != Some("helu") {
        return Ok(None);
    }

    // We just iterate all keys, which is fine for on-prem where api_keys table is < 100 keys
    let keys = sqlx::query!(
        "SELECT id, key_hash FROM api_keys WHERE enabled = TRUE"
    )
    .fetch_all(pool)
    .await?;

    let argon2 = Argon2::default();

    for record in keys {
        if let Ok(parsed_hash) = PasswordHash::new(&record.key_hash) {
            if argon2.verify_password(key.as_bytes(), &parsed_hash).is_ok() {
                // Update last used
                let _ = sqlx::query!(
                    "UPDATE api_keys SET last_used = NOW() WHERE id = $1",
                    record.id
                )
                .execute(pool)
                .await;

                return Ok(Some(record.id));
            }
        }
    }

    Ok(None)
}

pub fn hash_api_key(key: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(key.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash API key: {}", e))?
        .to_string();
    Ok(hash)
}

pub fn generate_raw_key() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    let random_part: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    format!("helu_{}", random_part)
}

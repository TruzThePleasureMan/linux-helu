use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::middleware::auth::AuthContext;
use crate::error::AppError;
use crate::auth::apikey::{generate_raw_key, hash_api_key};

#[derive(Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateKeyResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub key: String,
    pub created_at: String,
}

pub async fn create_api_key(
    State(state): State<AppState>,
    _ctx: AuthContext,
    Json(payload): Json<CreateKeyRequest>,
) -> Result<Json<CreateKeyResponse>, AppError> {
    let raw_key = generate_raw_key();
    let key_hash = hash_api_key(&raw_key)?;

    let record = sqlx::query!(
        "INSERT INTO api_keys (name, key_hash) VALUES ($1, $2) RETURNING id, created_at",
        payload.name,
        key_hash
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(CreateKeyResponse {
        id: record.id,
        name: payload.name,
        key: raw_key,
        created_at: record.created_at.format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default(),
    }))
}

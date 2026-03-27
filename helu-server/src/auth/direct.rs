use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use crate::middleware::auth::AuthContext;
use crate::AppState;
use crate::error::AppError;
use tracing::info;

#[derive(Deserialize)]
pub struct DirectAuthRequest {
    pub username: String,
    pub method: String,
    pub credential: String,
}

#[derive(Serialize)]
pub struct DirectAuthResponse {
    pub token: String,
    pub expires_at: String,
    pub method: String,
    pub username: String,
}

pub async fn direct_auth(
    State(state): State<AppState>,
    ctx: AuthContext,
    Json(payload): Json<DirectAuthRequest>,
) -> Result<Json<DirectAuthResponse>, AppError> {
    info!("Direct auth request for {} via {}", payload.username, payload.method);

    if payload.method != "pin" {
        return Err(AppError::BadRequest("Only 'pin' is supported for direct auth".to_string()));
    }

    let (success, reason) = state.dbus_client.trigger_auth_with_credential(
        &payload.username,
        &payload.method,
        &payload.credential
    ).await?;

    let client_ip = ctx.ip;
    sqlx::query!(
        r#"
        INSERT INTO auth_log (username, method, success, reason, client_ip, api_key_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        payload.username,
        payload.method,
        success,
        reason,
        client_ip,
        ctx.api_key_id
    )
    .execute(&state.db_pool)
    .await?;

    if !success {
        return Err(AppError::Unauthorized(reason));
    }

    let token = state.token_manager.issue_token(&payload.username, &payload.method)?;
    let decoded = state.token_manager.verify_token(&token)?;

    Ok(Json(DirectAuthResponse {
        token,
        expires_at: time::OffsetDateTime::from_unix_timestamp(decoded.exp)
            .unwrap_or(time::OffsetDateTime::now_utc())
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default(),
        method: decoded.method,
        username: decoded.sub,
    }))
}

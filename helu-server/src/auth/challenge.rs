use base64::Engine;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use crate::middleware::auth::AuthContext;
use crate::AppState;
use crate::error::AppError;
use tracing::info;

#[derive(Deserialize)]
pub struct ChallengeRequest {
    pub username: String,
    pub method: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub nonce: String,
    pub expires_at: String,
    pub method: String,
}

pub async fn issue_challenge(
    State(state): State<AppState>,
    _ctx: AuthContext,
    Json(payload): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, AppError> {
    info!("Issuing challenge for user {} via {}", payload.username, payload.method);

    // Resolve method
    let method = if payload.method == "auto" {
        "face".to_string() // In a real system, you might ask D-Bus for the user's default method, but for now we default to face for "auto"
    } else {
        if !state.config.auth.allowed_methods.contains(&payload.method) {
            return Err(AppError::BadRequest(format!("Method {} not allowed", payload.method)));
        }
        payload.method.clone()
    };

    use rand::{distributions::Alphanumeric, Rng};
    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let now = time::OffsetDateTime::now_utc();
    let expires_at = now + time::Duration::seconds(state.config.challenge.ttl_secs as i64);

    let challenge_id = sqlx::query!(
        r#"
        INSERT INTO challenges (username, nonce, issued_at, expires_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        payload.username,
        nonce,
        now,
        expires_at
    )
    .fetch_one(&state.db_pool)
    .await?
    .id;

    // Trigger UI on auth node
    // We ignore failure here so we can still return the challenge to the client
    // The client will poll /auth/verify, which will trigger the actual wait
    // Wait, the specification says:
    // 4. helu-server calls helud on the auth node via D-Bus:
    // 5. Client polls /auth/verify with challenge_id
    // But then: "1. /auth/challenge — store challenge in DB, return challenge_id"
    // "2. /auth/verify — call helud.Authenticate(username, method) and await result"
    // So we don't call D-Bus in `challenge` anymore based on clarification.

    Ok(Json(ChallengeResponse {
        challenge_id: challenge_id.to_string(),
        nonce: base64::prelude::BASE64_STANDARD.encode(nonce.as_bytes()),
        expires_at: expires_at.format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default(),
        method,
    }))
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub challenge_id: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub token: String,
    pub expires_at: String,
    pub method: String,
    pub username: String,
}

pub async fn verify_challenge(
    State(state): State<AppState>,
    ctx: AuthContext,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    info!("Verifying challenge {} for {}", payload.challenge_id, payload.username);
    let cid = uuid::Uuid::parse_str(&payload.challenge_id)
        .map_err(|_| AppError::BadRequest("Invalid challenge_id format".to_string()))?;

    let challenge = sqlx::query!(
        "SELECT id, expires_at, used FROM challenges WHERE id = $1 AND username = $2",
        cid,
        payload.username
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Challenge not found".to_string()))?;

    if challenge.used {
        return Err(AppError::Unauthorized("Challenge already used".to_string()));
    }

    if time::OffsetDateTime::now_utc() > challenge.expires_at {
        return Err(AppError::Unauthorized("challenge_expired".to_string()));
    }

    // Attempt D-Bus auth - blocks until UI completes
    let (success, reason) = state.dbus_client.trigger_auth(&payload.username, "auto").await?;

    // Log auth
    let client_ip = ctx.ip;
    sqlx::query!(
        r#"
        INSERT INTO auth_log (username, method, success, reason, client_ip, api_key_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        payload.username,
        "auto",
        success,
        reason,
        client_ip,
        ctx.api_key_id
    )
    .execute(&state.db_pool)
    .await?;

    // Mark used regardless of success to prevent reuse of nonce
    sqlx::query!(
        "UPDATE challenges SET used = TRUE WHERE id = $1",
        cid
    )
    .execute(&state.db_pool)
    .await?;

    if !success {
        return Err(AppError::Unauthorized(reason));
    }

    // Issue JWT
    let token = state.token_manager.issue_token(&payload.username, "auto")?;
    let decoded = state.token_manager.verify_token(&token)?;

    Ok(Json(VerifyResponse {
        token,
        expires_at: time::OffsetDateTime::from_unix_timestamp(decoded.exp)
            .unwrap_or(time::OffsetDateTime::now_utc())
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default(),
        method: decoded.method,
        username: decoded.sub,
    }))
}

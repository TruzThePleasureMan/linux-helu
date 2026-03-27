use axum::{Json, extract::{State, Path}};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::middleware::auth::AuthContext;
use crate::error::AppError;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub username: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

pub async fn list_users(
    State(state): State<AppState>,
    _ctx: AuthContext,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, enabled FROM users"
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(Json(users))
}

pub async fn create_user(
    State(state): State<AppState>,
    _ctx: AuthContext,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = sqlx::query_as!(
        UserResponse,
        "INSERT INTO users (username, enabled) VALUES ($1, $2) RETURNING id, username, enabled",
        payload.username,
        payload.enabled
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(user))
}

pub async fn disable_user(
    State(state): State<AppState>,
    _ctx: AuthContext,
    Path(username): Path<String>,
) -> Result<Json<()>, AppError> {
    sqlx::query!(
        "UPDATE users SET enabled = FALSE WHERE username = $1",
        username
    )
    .execute(&state.db_pool)
    .await?;

    Ok(Json(()))
}

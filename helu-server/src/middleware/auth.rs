use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::error::AppError;
use crate::AppState;
use ipnetwork::IpNetwork;
use std::net::IpAddr;

#[derive(Clone)]
pub struct AuthContext {
    pub api_key_id: Option<uuid::Uuid>,
    pub ip: Option<IpNetwork>,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthContext {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();

        let api_key = if auth_header.starts_with("Bearer ") {
            auth_header.trim_start_matches("Bearer ").trim()
        } else {
            return Err(AppError::Unauthorized("Missing or invalid Authorization header".to_string()));
        };

        let key_id = crate::auth::apikey::verify_api_key(&state.db_pool, api_key)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()))?;

        // Extract client IP
        let ip_addr = parts.extensions.get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|c| c.0.ip());

        let ip = ip_addr.map(|addr| match addr {
            IpAddr::V4(v4) => IpNetwork::V4(ipnetwork::Ipv4Network::new(v4, 32).unwrap()),
            IpAddr::V6(v6) => IpNetwork::V6(ipnetwork::Ipv6Network::new(v6, 128).unwrap()),
        });

        Ok(AuthContext {
            api_key_id: Some(key_id),
            ip,
        })
    }
}

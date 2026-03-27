pub mod config;
pub mod error;
pub mod db;
pub mod dbus;
pub mod auth;
pub mod admin;
pub mod middleware;

use std::sync::Arc;
use axum::{
    routing::{get, post, delete},
    Router, Json,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use serde::Serialize;
use anyhow::Context;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub dbus_client: Arc<dbus::HeluDbusClient>,
    pub token_manager: Arc<auth::jwt::TokenManager>,
    pub config: Arc<config::HeluServerConfig>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "helu_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting helu-server...");

    let conf = config::HeluServerConfig::load("/etc/helu/helu-server.toml").unwrap_or_else(|e| {
        tracing::warn!("Failed to load config file: {}. Using default configuration.", e);
        config::HeluServerConfig::load("dummy").unwrap()
    });

    let db_pool = db::setup_db(&conf.database.url, conf.database.max_connections).await?;
    let dbus_client = dbus::HeluDbusClient::new(&conf.dbus.bus).await.context("Failed to connect to D-Bus")?;
    let token_manager = auth::jwt::TokenManager::new(&conf.jwt).context("Failed to setup JWT Manager")?;

    let state = AppState {
        db_pool,
        dbus_client: Arc::new(dbus_client),
        token_manager: Arc::new(token_manager),
        config: Arc::new(conf.clone()),
    };

    let app = Router::new()
        .route("/health", get(|| async {
            Json(HealthResponse {
                status: "ok".to_string(),
                version: "0.1.0".to_string(),
            })
        }))
        .route("/auth/challenge", post(auth::challenge::issue_challenge))
        .route("/auth/verify", post(auth::challenge::verify_challenge))
        .route("/auth/direct", post(auth::direct::direct_auth))
        .route("/admin/users", get(admin::users::list_users).post(admin::users::create_user))
        .route("/admin/users/:username", delete(admin::users::disable_user))
        .route("/admin/api-keys", post(admin::keys::create_api_key))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&conf.server.bind).await.unwrap();
    tracing::info!("Listening on {}", conf.server.bind);

    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();

    Ok(())
}

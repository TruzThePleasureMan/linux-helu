use serde::Deserialize;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub port: u16,
    pub jwt_secret: String,
    pub token_ttl: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_string(),
            port: 8080,
            jwt_secret: "change-me-in-production".to_string(),
            token_ttl: 3600,
        }
    }
}

impl ServerConfig {
    pub fn load() -> Result<Self> {
        // Skeleton loader
        let path = Path::new("/etc/helu/helu-server.toml");
        if path.exists() {
            // Load from file in real implementation
            Ok(Self::default())
        } else {
            Ok(Self::default())
        }
    }
}

use serde::Deserialize;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind: String,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: Option<String>,
    pub ttl_secs: u64,
    pub algorithm: String,
    pub private_key_path: Option<String>,
    pub public_key_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeConfig {
    pub ttl_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbusConfig {
    pub bus: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub allowed_methods: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HeluServerConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub challenge: ChallengeConfig,
    pub dbus: DbusConfig,
    pub auth: AuthConfig,
}

impl HeluServerConfig {
    pub fn load(path_str: &str) -> Result<Self> {
        let path = Path::new(path_str);

        let contents = if path.exists() {
            fs::read_to_string(path).context(format!("Failed to read config file at {}", path_str))?
        } else {
            // Provide a default dev configuration if none exists
            String::from(r#"
[server]
bind = "0.0.0.0:8080"

[database]
url = "postgres://helu:password@localhost/helu"
max_connections = 10

[jwt]
secret = "change-me-in-production"
ttl_secs = 3600
algorithm = "HS256"

[challenge]
ttl_secs = 60

[dbus]
bus = "session"

[auth]
allowed_methods = ["face", "fingerprint", "fido2", "pin"]
            "#)
        };

        let config: HeluServerConfig = toml::from_str(&contents).context("Failed to parse config file")?;

        if config.jwt.algorithm == "RS256" {
            let priv_path = config.jwt.private_key_path.as_ref().context("helu-server: RS256 configured but private_key_path not set")?;
            let pub_path = config.jwt.public_key_path.as_ref().context("helu-server: RS256 configured but public_key_path not set")?;

            if !Path::new(priv_path).exists() {
                anyhow::bail!("helu-server: RS256 configured but private key not found at {}", priv_path);
            }
            if !Path::new(pub_path).exists() {
                anyhow::bail!("helu-server: RS256 configured but public key not found at {}", pub_path);
            }
        } else if config.jwt.algorithm == "HS256" && config.jwt.secret.is_none() {
             anyhow::bail!("helu-server: HS256 configured but secret not set");
        }

        Ok(config)
    }
}

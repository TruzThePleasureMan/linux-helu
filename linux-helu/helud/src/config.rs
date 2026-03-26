use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub face: FaceConfig,
    #[serde(default)]
    pub fingerprint: FingerprintConfig,
    #[serde(default)]
    pub pin: PinConfig,
    #[serde(default)]
    pub fido2: Fido2Config,
    #[serde(default)]
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemon: DaemonConfig::default(),
            face: FaceConfig::default(),
            fingerprint: FingerprintConfig::default(),
            pin: PinConfig::default(),
            fido2: Fido2Config::default(),
            ui: UiConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaemonConfig {
    pub bus: String, // "session" or "system"
    pub socket: String,
    pub log_level: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            bus: "session".to_string(),
            socket: "/run/helu/helu.sock".to_string(),
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FaceConfig {
    pub enabled: bool,
    pub model_path: String,
    pub threshold: f32,
    pub camera_index: u32,
    pub mock_hardware: bool,
}

impl Default for FaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model_path: "/usr/share/helu/models/mobilefacenet.onnx".to_string(),
            threshold: 0.6,
            camera_index: 0,
            mock_hardware: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FingerprintConfig {
    pub enabled: bool,
    pub mock_hardware: bool,
}

impl Default for FingerprintConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mock_hardware: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PinConfig {
    pub enabled: bool,
    pub min_length: usize,
}

impl Default for PinConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_length: 4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fido2Config {
    pub enabled: bool,
    pub mock_hardware: bool,
}

impl Default for Fido2Config {
    fn default() -> Self {
        Self {
            enabled: true,
            mock_hardware: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub accent_color: String,
    pub greeting: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            accent_color: "#e95420".to_string(),
            greeting: "Helu".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let system_path = Path::new("/etc/helu/helu.toml");
        let mut config = if system_path.exists() {
            let content = std::fs::read_to_string(system_path)?;
            toml::from_str(&content).context("Failed to parse system config")?
        } else {
            Config::default()
        };

        if let Some(proj_dirs) = ProjectDirs::from("net", "helu", "helu") {
            let user_path = proj_dirs.config_dir().join("helu.toml");
            if user_path.exists() {
                let content = std::fs::read_to_string(&user_path)?;
                let user_config: Config = toml::from_str(&content).context("Failed to parse user config")?;

                // Simple merge - user overrides system
                config = user_config;
            }
        }

        Ok(config)
    }
}

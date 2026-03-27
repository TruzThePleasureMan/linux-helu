use super::AuthMethod;
use anyhow::Result;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[allow(dead_code)]
pub struct PinAuth {
    config: crate::config::PinConfig,
}

impl PinAuth {
    pub fn new(config: crate::config::PinConfig) -> Self {
        Self { config }
    }

    fn pin_file_path(&self, username: &str) -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("net", "helu", "helu") {
            let user_dir = proj_dirs.data_local_dir().join("pins");
            fs::create_dir_all(&user_dir)?;
            Ok(user_dir.join(format!("{}.pin", username)))
        } else {
            anyhow::bail!("Failed to get project directories")
        }
    }

    fn hash_pin(&self, pin: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

impl AuthMethod for PinAuth {
    fn name(&self) -> &'static str {
        "pin"
    }

    fn authenticate(&self, username: &str) -> Result<bool> {
        let path = self.pin_file_path(username)?;
        if !path.exists() {
            return Ok(false);
        }

        let stored_hash = fs::read_to_string(&path)?.trim().to_string();

        // Mock implementation for now:
        tracing::info!("PIN auth requested for {}", username);
        if let Ok(env_pin) = std::env::var("HELU_MOCK_PIN") {
            return Ok(self.hash_pin(&env_pin) == stored_hash);
        }

        Ok(false)
    }

    fn enroll(&mut self, username: &str) -> Result<bool> {
        let path = self.pin_file_path(username)?;

        // Mock enrollment
        if let Ok(env_pin) = std::env::var("HELU_MOCK_PIN") {
            let hash = self.hash_pin(&env_pin);
            fs::write(path, hash)?;
            return Ok(true);
        }

        Ok(false)
    }
}

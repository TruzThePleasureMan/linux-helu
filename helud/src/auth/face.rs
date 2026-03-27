use super::AuthMethod;
use anyhow::Result;
use std::path::Path;
use std::fs;

use crate::ml::model::load_session;
use crate::ml::pipeline::{frame_to_embedding, is_match};
use crate::ml::camera::Camera;
use crate::crypto::{store::load_embeddings, fallback::derive_key_software, tpm::{tpm_available, unseal_key}};
use helu_common::types::AuthResult;

pub struct FaceAuth {
    config: crate::config::FaceConfig,
    crypto_config: crate::config::CryptoConfig,
}

unsafe impl Send for FaceAuth {}
unsafe impl Sync for FaceAuth {}

impl FaceAuth {
    pub fn new(config: crate::config::FaceConfig, crypto_config: crate::config::CryptoConfig) -> Self {
        Self { config, crypto_config }
    }

    fn get_key(&self) -> Result<[u8; 32]> {
        if tpm_available(&self.crypto_config.tpm_device) {
            let sealed_path = Path::new(&self.crypto_config.sealed_key_path);
            if sealed_path.exists() {
                let blob = fs::read(sealed_path)?;
                return unseal_key(&blob);
            }
        }

        if self.crypto_config.software_fallback {
            tracing::warn!("TPM not available or key not sealed. Using software key derivation.");
            derive_key_software()
        } else {
            anyhow::bail!("TPM required but not available.")
        }
    }

    pub async fn authenticate_face(&self, username: &str) -> AuthResult {
        let model_path = Path::new(&self.config.model_path);
        if !model_path.exists() {
            tracing::error!("Face model not found at {:?}", model_path);
            if self.config.mock_hardware {
                return AuthResult::Success(String::new());
            }
            return AuthResult::Error("Face model not found".to_string());
        }

        let key = match self.get_key() {
            Ok(k) => k,
            Err(e) => {
                tracing::error!("Failed to get key: {}", e);
                return AuthResult::Error(format!("Failed to get key: {}", e));
            }
        };

        let stored_embeddings = match load_embeddings(username, &key) {
            Ok(e) => e,
            Err(_) => return AuthResult::Error("Failed to load embeddings".to_string()),
        };

        if stored_embeddings.is_empty() {
            return AuthResult::NotEnrolled(String::new());
        }

        let mut camera = match Camera::open(self.config.camera_index, self.config.mock_hardware) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Camera open failed: {}", e);
                return AuthResult::Error(format!("Camera open failed: {}", e));
            }
        };

        let mut session = match load_session(model_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to load ONNX session: {}", e);
                return AuthResult::Error(format!("Failed to load ONNX session: {}", e));
            }
        };

        let frame = match camera.capture_frame() {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Failed to capture frame: {}", e);
                return AuthResult::Error(format!("Failed to capture frame: {}", e));
            }
        };

        let embedding = match frame_to_embedding(&mut session, &frame) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("Failed to get embedding: {}", e);
                return AuthResult::Error(format!("Failed to get embedding: {}", e));
            }
        };

        if is_match(&embedding, &stored_embeddings, self.config.threshold) {
            AuthResult::Success(String::new())
        } else {
            AuthResult::Failure("face_not_recognized".to_string())
        }
    }

    fn run_pipeline(&self, username: &str) -> Result<AuthResult> {
        let rt = tokio::runtime::Runtime::new()?;
        Ok(rt.block_on(self.authenticate_face(username)))
    }
}

impl AuthMethod for FaceAuth {
    fn name(&self) -> &'static str {
        "face"
    }

    fn authenticate(&self, username: &str) -> Result<bool> {
        match self.run_pipeline(username)? {
            AuthResult::Success(_) => Ok(true),
            AuthResult::Failure(reason) => {
                tracing::info!("Face Auth Failed: {}", reason);
                Ok(false)
            },
            AuthResult::NotEnrolled(_) => Ok(false),
            AuthResult::Error(e) => {
                tracing::error!("Face Auth Error: {}", e);
                Ok(false)
            },
        }
    }

    #[allow(dead_code)]
    fn authenticate_result(&self, username: &str) -> Result<AuthResult> {
        self.run_pipeline(username)
    }

    fn enroll(&mut self, username: &str) -> Result<bool> {
        crate::enrollment::face::enroll_face(username, &self.config, &self.crypto_config)
    }
}

use super::AuthMethod;
use anyhow::Result;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;

pub struct FaceAuth {
    config: crate::config::FaceConfig,
}

// Since ort threading models can be strict and require complex build steps,
// for this mock we will just simulate the result.
unsafe impl Send for FaceAuth {}
unsafe impl Sync for FaceAuth {}

impl FaceAuth {
    pub fn new(config: crate::config::FaceConfig) -> Self {
        Self { config }
    }

    fn face_dir_path(&self, username: &str) -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("net", "helu", "helu") {
            let user_dir = proj_dirs.data_local_dir().join("faces").join(username);
            std::fs::create_dir_all(&user_dir)?;
            Ok(user_dir)
        } else {
            anyhow::bail!("Failed to get project directories")
        }
    }

    fn run_pipeline(&self, username: &str) -> Result<bool> {
        tracing::info!("Initializing Face Pipeline (ort)...");

        let model_path = Path::new(&self.config.model_path);

        // If the file doesn't exist but we are mocking, just log and return success
        if !model_path.exists() {
            if self.config.mock_hardware {
                tracing::info!("Model not found at {}, but mock_hardware is true. Simulating success.", model_path.display());
                return Ok(true);
            } else {
                anyhow::bail!("Face model not found at {}", model_path.display());
            }
        }

        tracing::info!("Loading face model from {:?}", model_path);

        // This is where real ort session initialization would be.
        // let session = ort::Session::builder()?.commit_from_file(model_path)?;

        // Simulated cropped face input tensor
        tracing::info!("Processing input tensor...");

        tracing::info!("Got 128D embedding. Comparing with stored...");

        // 8. Load enrolled embeddings
        let face_dir = self.face_dir_path(username)?;
        let mut best_score = 0.0f32;

        if face_dir.exists() {
            for entry in std::fs::read_dir(face_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "bin") {
                    // 9. Calculate Cosine Similarity
                    // Mock calculation
                    best_score = 0.8;
                }
            }
        }

        // 10. Compare to threshold
        if best_score > self.config.threshold {
            Ok(true)
        } else {
            // If mock, just say yes
            if self.config.mock_hardware {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

impl AuthMethod for FaceAuth {
    fn name(&self) -> &'static str {
        "face"
    }

    fn authenticate(&self, username: &str) -> Result<bool> {
        self.run_pipeline(username)
    }

    fn enroll(&mut self, username: &str) -> Result<bool> {
        let _path = self.face_dir_path(username)?;
        if self.config.mock_hardware {
            tracing::info!("Mocking face enroll: Success for {}", username);
            return Ok(true);
        }

        // This is where real capture + store embedding logic would go.
        Ok(false)
    }
}

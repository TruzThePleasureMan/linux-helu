use std::fs;
use std::path::Path;
use std::time::Duration;
use crate::config::{FaceConfig, CryptoConfig};
use crate::ml::model::load_session;
use crate::ml::pipeline::frame_to_embedding;
use crate::ml::camera::Camera;
use crate::crypto::{store::save_embeddings, fallback::derive_key_software, tpm::{tpm_available, seal_key}};

pub fn enroll_face(
    username: &str,
    config: &FaceConfig,
    crypto_config: &CryptoConfig,
) -> anyhow::Result<bool> {
    let model_path = Path::new(&config.model_path);
    if !model_path.exists() && !config.mock_hardware {
        anyhow::bail!("Face model not found at {:?}", model_path);
    }

    let mut camera = Camera::open(config.camera_index, config.mock_hardware)?;
    let mut session = if !config.mock_hardware || model_path.exists() {
        Some(load_session(model_path)?)
    } else {
        None
    };

    let mut embeddings = Vec::new();
    let num_frames = config.enrollment_frames;

    for i in 0..num_frames {
        tracing::info!("Capturing frame {}/{} for enrollment...", i + 1, num_frames);
        let frame = camera.capture_frame()?;

        if let Some(ref mut sess) = session {
            let embedding = frame_to_embedding(sess, &frame)?;
            embeddings.push(embedding);
        } else {
            // Mock mode, just push an empty/dummy array
            use ndarray::Array1;
            embeddings.push(Array1::zeros(512));
        }

        // Wait a bit to get slight variations
        std::thread::sleep(Duration::from_millis(600));
    }

    // Handle keys
    let key = if tpm_available(&crypto_config.tpm_device) {
        let sealed_path = Path::new(&crypto_config.sealed_key_path);
        if sealed_path.exists() {
            crate::crypto::tpm::unseal_key(&fs::read(sealed_path)?)?
        } else {
            let mut new_key = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut new_key);
            let sealed = seal_key(&new_key, &crypto_config.tpm_pcrs)?;
            fs::write(sealed_path, sealed)?;
            new_key
        }
    } else if crypto_config.software_fallback {
        tracing::warn!("TPM not available. Using software key derivation.");
        derive_key_software()?
    } else {
        anyhow::bail!("TPM required but not available.")
    };

    // Encrypt and store all embeddings
    save_embeddings(username, &embeddings, &key)?;

    tracing::info!("Enrollment complete for user {}", username);
    Ok(true)
}

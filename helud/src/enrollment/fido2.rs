use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use ctap_hid_fido2::FidoKeyHid;
use rand::RngCore;
use std::fs;
use std::path::{Path, PathBuf};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};

use crate::crypto::tpm::{tpm_available, unseal_key};
use crate::crypto::fallback::derive_key_software;
use crate::config::Config;

#[derive(Serialize, Deserialize)]
pub struct Fido2Credential {
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,        // COSE-encoded
    pub device_aaguid: String,      // device identifier
    pub enrolled_at: DateTime<Utc>,
}

fn get_credential_path(username: &str) -> Result<PathBuf> {
    let dir_path = Path::new("/var/lib/helu/fido2").join(username);
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path)?;
    }
    Ok(dir_path.join("credential.json.enc"))
}

// Reuse the face crypto key logic locally for simplicity
fn get_encryption_key() -> Result<[u8; 32]> {
    let config = Config::load()?;
    let crypto = config.crypto;

    if tpm_available(&crypto.tpm_device) {
        let sealed_path = Path::new(&crypto.sealed_key_path);
        if sealed_path.exists() {
            let blob = fs::read(sealed_path)?;
            return unseal_key(&blob);
        }
    }

    if crypto.software_fallback {
        derive_key_software()
    } else {
        anyhow::bail!("TPM required but not available.")
    }
}

pub fn load_fido2_credential(username: &str) -> Result<Fido2Credential> {
    let path = get_credential_path(username)?;
    if !path.exists() {
        anyhow::bail!("Not enrolled");
    }

    let key = get_encryption_key()?;
    let data = fs::read(&path)?;

    if data.len() < 12 {
        anyhow::bail!("Invalid encrypted data");
    }

    let nonce = Nonce::from_slice(&data[0..12]);
    let ciphertext = &data[12..];

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let decrypted = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    let cred: Fido2Credential = serde_json::from_slice(&decrypted)?;
    Ok(cred)
}

pub fn save_fido2_credential(username: &str, cred: &Fido2Credential) -> Result<()> {
    let path = get_credential_path(username)?;
    let key = get_encryption_key()?;

    let serialized = serde_json::to_vec(cred)?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, serialized.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    let mut out_data = nonce_bytes.to_vec();
    out_data.extend(ciphertext);

    fs::write(&path, out_data)?;
    Ok(())
}

pub async fn enroll_fido2(username: &str) -> Result<()> {
    // 1. Generate random 32-byte user ID + challenge
    let mut user_id = vec![0u8; 32];
    let mut challenge = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut user_id);
    rand::thread_rng().fill_bytes(&mut challenge);

    let username_str = username.to_string();
    let enroll_task = tokio::task::spawn_blocking(move || {
        let devs = ctap_hid_fido2::get_fidokey_devices();
        let params: Vec<ctap_hid_fido2::HidParam> = devs.into_iter().map(|d| d.param).collect();
        let api_key = FidoKeyHid::new(&params, &ctap_hid_fido2::Cfg::init());

        let device = api_key.unwrap_or_else(|_| {
            let devs = ctap_hid_fido2::get_fidokey_devices();
            let params: Vec<ctap_hid_fido2::HidParam> = devs.into_iter().map(|d| d.param).collect();
            FidoKeyHid::new(&params, &ctap_hid_fido2::Cfg::init()).unwrap()
        });

        let rp = "net.helu.helud";

        let user_entity = ctap_hid_fido2::public_key_credential_user_entity::PublicKeyCredentialUserEntity::new(
            Some(&user_id),
            Some(&username_str),
            Some(&username_str)
        );

        device.make_credential_rk(rp, &challenge, None, &user_entity)
    });

    // 2. Send CTAP2 MakeCredential request (Prompts user to touch key)
    let make_credential_result = tokio::time::timeout(tokio::time::Duration::from_secs(30), enroll_task)
        .await
        .context("FIDO2 enrollment timed out after 30 seconds")?
        .context("FIDO2 enrollment task panicked")?;

    let attestation = make_credential_result.map_err(|e| anyhow::anyhow!("MakeCredential failed: {:?}", e))?;

    // ctap-hid-fido2 doesn't immediately expose the public key byte extraction trivially,
    // so we store the credential ID and mock the public key / AAGUID for now unless easily extracted.
    let cred_id = attestation.credential_descriptor.id;

    let cred = Fido2Credential {
        credential_id: cred_id,
        public_key: vec![], // In a fully complete binding we extract COSE key
        device_aaguid: "unknown".to_string(),
        enrolled_at: Utc::now(),
    };

    // 4. Store Fido2Credential as JSON at /var/lib/helu/fido2/{username}/credential.json
    save_fido2_credential(username, &cred)?;

    Ok(())
}

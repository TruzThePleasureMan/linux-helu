use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};
use rand::RngCore;

pub fn derive_key_software() -> anyhow::Result<[u8; 32]> {
    let machine_id_path = Path::new("/etc/machine-id");
    let machine_id = if machine_id_path.exists() {
        fs::read_to_string(machine_id_path)?.trim().to_string()
    } else {
        "fallback_machine_id".to_string()
    };

    let salt_path = Path::new("/var/lib/helu/salt");
    if !salt_path.exists() {
        if let Some(parent) = salt_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut random_salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut random_salt);
        fs::write(salt_path, random_salt)?;
    }

    let salt = fs::read(salt_path)?;

    // Simplified HKDF using Sha256 (for the purpose of this mock, just hashing the combo)
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(&salt);

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);

    Ok(key)
}

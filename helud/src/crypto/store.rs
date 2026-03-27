use std::fs;
use std::path::{Path, PathBuf};
use ndarray::Array1;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use rand::RngCore;

fn get_embedding_path(username: &str) -> anyhow::Result<PathBuf> {
    let dir_path = Path::new("/var/lib/helu/faces").join(username);
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path)?;
    }
    Ok(dir_path.join("embeddings.enc"))
}

pub fn save_embeddings(
    username: &str,
    embeddings: &[Array1<f32>],
    key: &[u8; 32],
) -> anyhow::Result<()> {
    let path = get_embedding_path(username)?;

    // Serialize embeddings: Vec of Vec<f32>
    let raw_vecs: Vec<Vec<f32>> = embeddings.iter().map(|e| e.to_vec()).collect();
    let serialized = postcard::to_stdvec(&raw_vecs)?;

    // Encrypt
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, serialized.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    // Prepend nonce to ciphertext
    let mut out_data = nonce_bytes.to_vec();
    out_data.extend(ciphertext);

    fs::write(&path, out_data)?;
    Ok(())
}

pub fn load_embeddings(
    username: &str,
    key: &[u8; 32],
) -> anyhow::Result<Vec<Array1<f32>>> {
    let path = get_embedding_path(username)?;
    if !path.exists() {
        return Ok(Vec::new()); // Not enrolled
    }

    let data = fs::read(&path)?;
    if data.len() < 12 {
        anyhow::bail!("Invalid encrypted data");
    }

    let nonce = Nonce::from_slice(&data[0..12]);
    let ciphertext = &data[12..];

    // Decrypt
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let decrypted = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    let raw_vecs: Vec<Vec<f32>> = postcard::from_bytes(&decrypted)?;
    let embeddings = raw_vecs.into_iter().map(Array1::from_vec).collect();

    Ok(embeddings)
}

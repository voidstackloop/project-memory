use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

const PBKDF2_ITERATIONS: u32 = 100_000;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// AES-256-GCM encrypt, keyed via PBKDF2. Output is hex(salt || nonce || ciphertext).
pub fn encrypt(content: &str, key: &str) -> String {
    if key.is_empty() { return content.to_string(); }
    let mut rng = rand::rngs::OsRng;
    let mut salt = [0u8; SALT_LEN];
    rng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill_bytes(&mut nonce_bytes);

    let derived = derive_key(key, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, content.as_bytes()).expect("encryption failure");

    let mut out = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    hex::encode(out)
}

pub fn decrypt(encoded: &str, key: &str) -> anyhow::Result<String> {
    if key.is_empty() { return Ok(encoded.to_string()); }
    let bytes = hex::decode(encoded).map_err(|e| anyhow::anyhow!("invalid encrypted content: {e}"))?;
    if bytes.len() < SALT_LEN + NONCE_LEN {
        anyhow::bail!("encrypted content is truncated");
    }
    let (salt, rest) = bytes.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

    let derived = derive_key(key, salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("decryption failed: wrong key or corrupted data"))?;
    Ok(String::from_utf8(plaintext)?)
}

pub fn get_encryption_key(project_dir: &std::path::Path) -> Option<String> {
    std::env::var("PMEM_ENCRYPTION_KEY").ok().filter(|k| !k.is_empty())
        .or_else(|| std::fs::read_to_string(project_dir.join(".memory").join(".key")).ok().map(|k| k.trim().to_string()).filter(|k| !k.is_empty()))
}
pub fn set_encryption_key(project_dir: &std::path::Path, key: &str) -> anyhow::Result<()> { std::fs::write(project_dir.join(".memory").join(".key"), key)?; Ok(()) }
pub fn remove_encryption_key(project_dir: &std::path::Path) -> anyhow::Result<()> { let p = project_dir.join(".memory").join(".key"); if p.exists() { std::fs::remove_file(p)?; } Ok(()) }

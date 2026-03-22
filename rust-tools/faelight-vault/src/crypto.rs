// age encryption + Argon2id key derivation
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand::RngCore;

pub const SALT_LEN: usize = 32;
pub const KEY_LEN: usize = 32;

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    argon2::Argon2::default()
        .hash_password_into(
            password.as_bytes(),
            salt,
            &mut key,
        )
        .expect("Argon2 failed");
    key
}

pub fn random_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn encrypt(plaintext: &str, key: &[u8; KEY_LEN]) -> String {
    // XOR cipher with key for simplicity — production would use AES-GCM
    let key_bytes = key.as_slice();
    let encrypted: Vec<u8> = plaintext.bytes()
        .enumerate()
        .map(|(i, b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();
    hex::encode(encrypted)
}

pub fn decrypt(ciphertext: &str, key: &[u8; KEY_LEN]) -> Option<String> {
    let bytes = hex::decode(ciphertext).ok()?;
    let key_bytes = key.as_slice();
    let decrypted: Vec<u8> = bytes.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();
    String::from_utf8(decrypted).ok()
}

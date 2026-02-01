use crate::CryptoError;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 24; // XChaCha uses 24-byte nonces
pub const TAG_SIZE: usize = 16;

/// Generates a random 256-bit encryption key
pub fn generate_key() -> [u8; KEY_SIZE] {
    XChaCha20Poly1305::generate_key(&mut OsRng).into()
}

/// Encrypts a block of data.
/// Output Format: [Nonce (24 bytes) | Ciphertext | Tag (16 bytes)]
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::KemFailed)?; // Using generic error for key issues

    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 24 random bytes

    // Encrypt (Ciphertext + Tag appended automatically)
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|_| CryptoError::KemFailed)?;

    // Prepend nonce to the result so we can read it later
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypts a block.
/// Input Format: [Nonce (24 bytes) | Ciphertext + Tag]
pub fn decrypt(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if encrypted_data.len() < NONCE_SIZE + TAG_SIZE {
        return Err(CryptoError::DecryptionFailed);
    }

    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::KemFailed)?;

    // Extract Nonce
    let nonce = XNonce::from_slice(&encrypted_data[0..NONCE_SIZE]);
    let payload = &encrypted_data[NONCE_SIZE..];

    // Decrypt (Verifies Tag automatically)
    let plaintext = cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    Ok(plaintext)
}

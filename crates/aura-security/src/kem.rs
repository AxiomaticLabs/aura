use pqcrypto_kyber::kyber1024; // Highest security level
use pqcrypto_traits::kem::{Ciphertext, SharedSecret, PublicKey, SecretKey};
use crate::CryptoError;

/// A wrapper for the User's Identity Keypair
pub struct PQCKeyPair {
    pub pk: kyber1024::PublicKey,
    pub sk: kyber1024::SecretKey,
}

impl PQCKeyPair {
    /// Step 1: Server generates a Keypair
    pub fn generate() -> Self {
        let (pk, sk) = kyber1024::keypair();
        Self { pk, sk }
    }
}

/// Step 2: Client Encapsulates (Creates the shared secret)
/// Returns: (The Secret to keep, The Ciphertext to send to Server)
pub fn encapsulate(pk_bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    // Deserialize the public key
    let pk = kyber1024::PublicKey::from_bytes(pk_bytes)
        .map_err(|_| CryptoError::KemFailed)?;

    // Encapsulate
    let (shared_secret, ciphertext) = kyber1024::encapsulate(&pk);
    
    Ok((shared_secret.as_bytes().to_vec(), ciphertext.as_bytes().to_vec()))
}

/// Step 3: Server Decapsulates (Recovers the shared secret)
pub fn decapsulate(ciphertext_bytes: &[u8], sk: &kyber1024::SecretKey) -> Result<Vec<u8>, CryptoError> {
    let ciphertext = kyber1024::Ciphertext::from_bytes(ciphertext_bytes)
        .map_err(|_| CryptoError::KemFailed)?;

    let shared_secret = kyber1024::decapsulate(&ciphertext, sk);
    Ok(shared_secret.as_bytes().to_vec())
}
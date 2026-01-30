pub mod homomorphic;
pub mod kem;
pub mod sign;
pub mod symmetric;
#[cfg(test)]
pub mod tests;

// Re-export KEM functions for convenience
pub use kem::{decapsulate, encapsulate, PQCKeyPair};

// Re-export common errors
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key Encapsulation Failed")]
    KemFailed,
    #[error("Signature Verification Failed")]
    InvalidSignature,
    #[error("Decryption Failed (Tag Mismatch)")]
    DecryptionFailed,
}

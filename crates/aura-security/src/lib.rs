pub mod kem;
pub mod sign;
pub mod homomorphic;
pub mod symmetric;
pub mod tests;

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
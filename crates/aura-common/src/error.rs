use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuraError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("Crypto Error: {0}")]
    Crypto(String),

    #[error("Key not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_creation_and_display() {
        // Test IO error
        let io_err = AuraError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        assert!(io_err.to_string().contains("IO Error"));
        assert!(io_err.to_string().contains("file not found"));

        // Test serialization error
        let ser_err = AuraError::Serialization("invalid data".to_string());
        assert!(ser_err.to_string().contains("Serialization Error"));
        assert!(ser_err.to_string().contains("invalid data"));

        // Test crypto error
        let crypto_err = AuraError::Crypto("decryption failed".to_string());
        assert!(crypto_err.to_string().contains("Crypto Error"));
        assert!(crypto_err.to_string().contains("decryption failed"));

        // Test not found error
        let not_found_err = AuraError::NotFound("user_123".to_string());
        assert!(not_found_err.to_string().contains("Key not found"));
        assert!(not_found_err.to_string().contains("user_123"));
    }

    #[test]
    fn test_error_from_conversion() {
        // Test From<io::Error> for AuraError
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let aura_error: AuraError = io_error.into();
        match aura_error {
            AuraError::Io(_) => {} // Correct conversion
            _ => panic!("Should convert to Io error"),
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        let err = AuraError::NotFound("test_key".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("test_key"));
    }
}

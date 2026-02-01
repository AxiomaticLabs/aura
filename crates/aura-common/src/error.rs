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

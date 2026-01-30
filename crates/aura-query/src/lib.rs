pub mod executor;
pub mod tests;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("SQL Parse Error: {0}")]
    Parse(#[from] sqlparser::parser::ParserError),
    #[error("Not Implemented: {0}")]
    Unimplemented(String),
    #[error("Storage Error: {0}")]
    Store(#[from] aura_store::StoreError),
    #[error("Serialization Error: {0}")]
    Serialization(String),
}

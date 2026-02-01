pub mod document;
pub mod error;

// Re-export commonly used types
pub use document::{AuraDocument, DataValue};
pub use error::AuraError;
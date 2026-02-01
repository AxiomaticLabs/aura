pub mod page;
pub mod pager;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Page not found: {0}")]
    PageNotFound(u32),
    #[error("Integrity Violation: Hash Mismatch on Page {0}")]
    Tampered(u32),
}

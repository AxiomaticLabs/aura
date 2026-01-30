use crate::StoreError;
use postcard;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A simple Primary Key Index.
/// Maps a String Key (e.g., "user_123") -> Page ID (e.g., 5).
#[derive(Debug, Serialize, Deserialize)]
pub struct PrimaryIndex {
    pub map: BTreeMap<String, u32>,
    pub dirty: bool, // Has the index changed since last save?
}

impl PrimaryIndex {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            dirty: false,
        }
    }

    pub fn insert(&mut self, key: String, page_id: u32) {
        self.map.insert(key, page_id);
        self.dirty = true;
    }

    pub fn get(&self, key: &str) -> Option<u32> {
        self.map.get(key).copied()
    }

    /// Serializes the entire index to bytes (to be saved in a Page)
    pub fn to_bytes(&self) -> Result<Vec<u8>, StoreError> {
        postcard::to_allocvec(&self.map)
            .map_err(|_| StoreError::Io(std::io::Error::other("Index serialization failed")))
    }

    /// Loads index from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StoreError> {
        let map: BTreeMap<String, u32> = postcard::from_bytes(bytes)
            .map_err(|_| StoreError::Io(std::io::Error::other("Index corruption")))?;

        Ok(Self { map, dirty: false })
    }
}

impl Default for PrimaryIndex {
    fn default() -> Self {
        Self::new()
    }
}

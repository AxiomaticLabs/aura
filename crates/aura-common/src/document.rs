use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The atomic unit of data in AuraDB.
/// This allows us to store SQL rows AND NoSQL JSON documents in the same engine.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Binary(Vec<u8>),

    /// A secure, opaque blob that can only be computed on via FHE
    Encrypted(Vec<u8>),

    /// For NoSQL arrays: ["tag1", "tag2"]
    Array(Vec<DataValue>),

    /// For NoSQL nested objects: {"address": {"city": "NY"}}
    Object(HashMap<String, DataValue>),
}

/// Represents a single Row (SQL) or Document (NoSQL).
/// This is what gets serialized and written to the Page.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuraDocument {
    /// Unique ID (Primary Key)
    pub id: String,

    /// Version for MVCC (Multi-Version Concurrency Control)
    pub version: u64,

    /// The actual data payload
    pub data: HashMap<String, DataValue>,
}

impl AuraDocument {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: 1,
            data: HashMap::new(),
        }
    }

    /// Serializes the document to compact binary format (Postcard)
    /// This is what we will encrypt and store on disk.
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Deserializes from binary format
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_document_serialization() {
        // 1. Create a "NoSQL" style document
        let mut doc = AuraDocument::new("user_123");

        doc.data
            .insert("name".to_string(), DataValue::Text("Alice".to_string()));
        doc.data.insert("age".to_string(), DataValue::Integer(30));

        // Nested object (impossible in standard SQL, easy here)
        let mut address = HashMap::new();
        address.insert(
            "city".to_string(),
            DataValue::Text("Wonderland".to_string()),
        );
        doc.data
            .insert("address".to_string(), DataValue::Object(address));

        // 2. Serialize to Binary (Simulate writing to disk)
        let bytes = doc.to_bytes().expect("Failed to serialize");
        println!("Serialized Size: {} bytes", bytes.len());

        // 3. Deserialize back (Simulate reading from disk)
        let loaded_doc = AuraDocument::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(doc.id, loaded_doc.id);

        // Verify the nested data exists
        match loaded_doc.data.get("address").unwrap() {
            DataValue::Object(map) => {
                assert_eq!(
                    map.get("city"),
                    Some(&DataValue::Text("Wonderland".to_string()))
                );
            }
            _ => panic!("Address should be an object"),
        }
    }
}

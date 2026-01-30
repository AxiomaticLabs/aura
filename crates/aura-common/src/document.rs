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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

    #[test]
    fn test_datavalue_variants() {
        // Test all DataValue variants
        assert_eq!(DataValue::Null, DataValue::Null);
        assert_eq!(DataValue::Boolean(true), DataValue::Boolean(true));
        assert_eq!(DataValue::Integer(42), DataValue::Integer(42));
        assert_eq!(
            DataValue::Float(std::f64::consts::PI),
            DataValue::Float(std::f64::consts::PI)
        );
        assert_eq!(
            DataValue::Text("hello".to_string()),
            DataValue::Text("hello".to_string())
        );
        assert_eq!(
            DataValue::Binary(vec![1, 2, 3]),
            DataValue::Binary(vec![1, 2, 3])
        );
        assert_eq!(
            DataValue::Encrypted(vec![4, 5, 6]),
            DataValue::Encrypted(vec![4, 5, 6])
        );
        assert_eq!(
            DataValue::Array(vec![DataValue::Integer(1), DataValue::Integer(2)]),
            DataValue::Array(vec![DataValue::Integer(1), DataValue::Integer(2)])
        );

        let mut obj = HashMap::new();
        obj.insert("key".to_string(), DataValue::Text("value".to_string()));
        assert_eq!(DataValue::Object(obj.clone()), DataValue::Object(obj));
    }

    #[test]
    fn test_document_creation_and_modification() {
        let mut doc = AuraDocument::new("test_id");
        assert_eq!(doc.id, "test_id");
        assert_eq!(doc.version, 1);
        assert!(doc.data.is_empty());

        // Test adding data
        doc.data
            .insert("name".to_string(), DataValue::Text("Alice".to_string()));
        doc.data.insert("age".to_string(), DataValue::Integer(25));
        doc.data
            .insert("active".to_string(), DataValue::Boolean(true));

        assert_eq!(doc.data.len(), 3);
        assert_eq!(
            doc.data.get("name"),
            Some(&DataValue::Text("Alice".to_string()))
        );
        assert_eq!(doc.data.get("age"), Some(&DataValue::Integer(25)));
        assert_eq!(doc.data.get("active"), Some(&DataValue::Boolean(true)));
    }

    #[test]
    fn test_document_serialization_edge_cases() {
        // Test empty document
        let empty_doc = AuraDocument::new("empty");
        let bytes = empty_doc.to_bytes().unwrap();
        let deserialized = AuraDocument::from_bytes(&bytes).unwrap();
        assert_eq!(empty_doc, deserialized);

        // Test document with all data types
        let mut full_doc = AuraDocument::new("full");
        full_doc
            .data
            .insert("null_val".to_string(), DataValue::Null);
        full_doc
            .data
            .insert("bool_val".to_string(), DataValue::Boolean(false));
        full_doc
            .data
            .insert("int_val".to_string(), DataValue::Integer(-123));
        full_doc.data.insert(
            "float_val".to_string(),
            DataValue::Float(-std::f64::consts::PI),
        );
        full_doc
            .data
            .insert("text_val".to_string(), DataValue::Text("".to_string())); // Empty string
        full_doc
            .data
            .insert("binary_val".to_string(), DataValue::Binary(vec![])); // Empty binary
        full_doc
            .data
            .insert("array_val".to_string(), DataValue::Array(vec![])); // Empty array

        let mut obj = HashMap::new();
        obj.insert("nested".to_string(), DataValue::Text("deep".to_string()));
        full_doc
            .data
            .insert("object_val".to_string(), DataValue::Object(obj));

        let bytes = full_doc.to_bytes().unwrap();
        let deserialized = AuraDocument::from_bytes(&bytes).unwrap();
        assert_eq!(full_doc, deserialized);
    }

    #[test]
    fn test_document_version_increment() {
        let mut doc = AuraDocument::new("version_test");
        assert_eq!(doc.version, 1);

        // Version should be manually incremented for MVCC
        doc.version = 2;
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_serialization_errors() {
        // Test deserialization with invalid data
        let invalid_bytes = vec![0, 1, 2, 3]; // Not valid postcard data
        assert!(AuraDocument::from_bytes(&invalid_bytes).is_err());

        // Test with empty bytes
        assert!(AuraDocument::from_bytes(&[]).is_err());
    }
}

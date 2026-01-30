use serde::{Deserialize, Serialize};

/// The Max size of a node (Must fit in Page - Metadata)
const NODE_CAPACITY: usize = 50;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum NodeType {
    Internal,
    Leaf,
}

/// A Single Node in the B+ Tree.
/// This gets serialized and encrypted into a Page.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BTreeNode {
    pub id: u32,             // The Page ID of this node
    pub parent: Option<u32>, // Parent Node ID (for traversing up)
    pub node_type: NodeType,

    /// The sorted keys in this node.
    /// In a real implementation, this would be generic bytes.
    /// For simplicity, we use String keys (e.g., "user_123").
    pub keys: Vec<String>,

    /// Pointers.
    /// If Internal: Points to Child Node IDs.
    /// If Leaf: Points to Data Page IDs.
    /// Note: children.len() is always keys.len() + 1 for Internal nodes.
    pub children: Vec<u32>,
}

impl BTreeNode {
    /// Creates a new empty Leaf Root
    pub fn new_leaf(id: u32) -> Self {
        Self {
            id,
            parent: None,
            node_type: NodeType::Leaf,
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn is_full(&self) -> bool {
        self.keys.len() >= NODE_CAPACITY
    }

    /// Serializes to fit in a 4KB Page
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

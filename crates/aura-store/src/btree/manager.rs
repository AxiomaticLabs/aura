use crate::btree::node::{BTreeNode, NodeType};
use crate::page::Page;
use crate::pager::Pager;
use crate::StoreError;
use std::io::Error;

pub struct BTreeManager<'a> {
    pager: &'a mut Pager,
    root_id: u32,
}

impl<'a> BTreeManager<'a> {
    /// Initialize the manager.
    /// If root_id is 0, it assumes a new tree needs to be created.
    pub fn new(pager: &'a mut Pager, root_id: u32) -> Self {
        Self { pager, root_id }
    }

    /// SEARCH: O(log n)
    /// Returns the Data Page ID for a given Key
    pub fn search(&mut self, key: &str) -> Result<Option<u32>, StoreError> {
        let mut current_id = self.root_id;

        loop {
            let node = self.read_node(current_id)?;

            match node.node_type {
                NodeType::Leaf => {
                    // Binary Search inside the Leaf
                    if let Ok(idx) = node.keys.binary_search(&key.to_string()) {
                        return Ok(Some(node.children[idx]));
                    } else {
                        return Ok(None);
                    }
                }
                NodeType::Internal => {
                    // Find which child to go down to
                    // find the first key strictly greater than 'key'
                    let idx = node.keys.partition_point(|k| k.as_str() <= key);
                    current_id = node.children[idx];
                }
            }
        }
    }

    /// INSERT: The complex part.
    /// For Step 9 Part 1, we will implement "Insert into Non-Full Node".
    /// Part 2 (Splitting) is a beast, we add that next.
    pub fn insert(&mut self, key: String, data_page_id: u32) -> Result<(), StoreError> {
        let root = self.read_node(self.root_id)?;

        if root.is_full() {
            // ROOT IS FULL: Tree grows in height

            // 1. Create a new Empty Root
            let new_root_id = self.pager.allocate_page();
            let mut new_root = BTreeNode {
                id: new_root_id,
                parent: None,
                node_type: NodeType::Internal, // Root is now Internal
                keys: Vec::new(),
                children: Vec::new(),
            };

            // 2. Make old root a child of new root
            new_root.children.push(self.root_id);

            // 3. Save new root temporarily so we can call split
            self.write_node(&new_root)?;

            // 4. Update old root's parent pointer
            let mut old_root = root; // 'root' variable from read_node
            old_root.parent = Some(new_root_id);
            self.write_node(&old_root)?;

            // 5. Split the old root (which is now child 0 of new root)
            self.split_child(&mut new_root, 0)?;

            // 6. Update the Manager's Root Pointer
            self.root_id = new_root_id;

            // 7. Finally insert the data into the new structure
            self.insert_non_full(new_root_id, key, data_page_id)?;
        } else {
            // Normal insert
            self.insert_non_full(self.root_id, key, data_page_id)?;
        }
        Ok(())
    }

    fn insert_non_full(&mut self, node_id: u32, key: String, value: u32) -> Result<(), StoreError> {
        let mut node = self.read_node(node_id)?;

        match node.node_type {
            NodeType::Leaf => {
                // Insert sorted
                let idx = node.keys.partition_point(|k| k < &key);
                node.keys.insert(idx, key);
                node.children.insert(idx, value);
                self.write_node(&node)?;
            }
            NodeType::Internal => {
                // Find child index
                let mut idx = node.keys.partition_point(|k| k < &key);

                // CHECK IF CHILD IS FULL BEFORE DESCENDING
                let child_id = node.children[idx];
                let child = self.read_node(child_id)?;

                if child.is_full() {
                    // PREEMPTIVE SPLIT
                    self.split_child(&mut node, idx)?;

                    // After split, the middle key moved up to 'node'.
                    // We must decide which of the two new children to descend into.
                    if key > node.keys[idx] {
                        idx += 1;
                    }
                }

                self.insert_non_full(node.children[idx], key, value)?;
            }
        }
        Ok(())
    }

    // --- HELPER: Read/Write Nodes using the Encrypted Pager ---

    fn read_node(&mut self, node_id: u32) -> Result<BTreeNode, StoreError> {
        let page = self.pager.read_page(node_id)?;
        let bytes = &page.data[..page.used_space as usize];
        let node = BTreeNode::from_bytes(bytes).map_err(|_| {
            StoreError::Io(Error::new(std::io::ErrorKind::InvalidData, "Node Corrupt"))
        })?;
        Ok(node)
    }

    fn write_node(&mut self, node: &BTreeNode) -> Result<(), StoreError> {
        let bytes = node
            .to_bytes()
            .map_err(|_| StoreError::Io(std::io::Error::other("Serialize Fail")))?;

        let mut page = Page::new(node.id);

        if bytes.len() > crate::page::DATA_SIZE {
            return Err(StoreError::Io(std::io::Error::other(
                "Node too big for Page",
            )));
        }

        page.used_space = bytes.len() as u16;
        page.data[..bytes.len()].copy_from_slice(&bytes);

        // This automatically Encrypts it!
        self.pager.write_page(&page)?;
        Ok(())
    }

    /// SPLIT CHILD: The core balancing algorithm.
    /// Splits child node at `child_idx` of `parent`.
    fn split_child(&mut self, parent: &mut BTreeNode, child_idx: usize) -> Result<(), StoreError> {
        let child_id = parent.children[child_idx];
        let mut child = self.read_node(child_id)?;

        // 1. Create a new Sibling Node (z)
        let new_page_id = self.pager.allocate_page();
        let mut sibling = BTreeNode {
            id: new_page_id,
            parent: Some(parent.id),
            node_type: child.node_type.clone(),
            keys: Vec::new(),
            children: Vec::new(),
        };

        // 2. Determine Split Point (Midpoint)
        let mid = child.keys.len() / 2;

        // 3. Move Right Half Keys to Sibling
        // Drain returns an iterator that removes items from 'child'
        let right_keys: Vec<String> = child.keys.drain(mid..).collect();

        // Handling Logic differs for Leaf vs Internal
        match child.node_type {
            NodeType::Leaf => {
                // In B+ Trees, Leaves contain ALL data.
                // So the split key is COPIED to parent, but STAYS in the leaf (usually).
                // Simplified strategy: Move right half.
                sibling.keys = right_keys;

                // Move Children (Data Pointers) as well
                let right_children: Vec<u32> = child.children.drain(mid..).collect();
                sibling.children = right_children;
            }
            NodeType::Internal => {
                // For Internal nodes, the middle key MOVES UP (is removed from child)
                // The first key of the right half is the one that moves up.
                // We skip the first key in the sibling because it goes to parent.
                // (Note: This is a simplified split for clarity. Real implementations are fiddly here.)

                // Let's stick to standard B-Tree logic for robustness:
                // Move keys [mid + 1 ..] to sibling
                // Key [mid] moves to parent

                // We essentially need to pop the key that goes to parent first
                // But since we drained `mid..`, that key is now sibling.keys[0]

                sibling.keys = right_keys;
                let key_to_parent = sibling.keys.remove(0); // Take the first one

                // Move corresponding children
                let right_children: Vec<u32> = child.children.drain((mid + 1)..).collect();
                sibling.children = right_children;

                // Update parent pointers of the children we just moved!
                // They used to point to 'child', now they must point to 'sibling'
                for &grandchild_id in &sibling.children {
                    let mut grandchild = self.read_node(grandchild_id)?;
                    grandchild.parent = Some(sibling.id);
                    self.write_node(&grandchild)?;
                }

                // Insert Key to Parent
                parent.keys.insert(child_idx, key_to_parent);
            }
        }

        // For Leaf Split, we usually copy the first key of sibling to parent
        if let NodeType::Leaf = child.node_type {
            parent.keys.insert(child_idx, sibling.keys[0].clone());
        }

        // 4. Hook up the Sibling to Parent
        parent.children.insert(child_idx + 1, sibling.id);

        // 5. Commit Changes to Disk
        self.write_node(&child)?;
        self.write_node(&sibling)?;
        self.write_node(parent)?;

        Ok(())
    }
}

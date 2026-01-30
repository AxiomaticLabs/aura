#[cfg(test)]
use crate::{
    page::Page,
    pager::{Pager, ENCRYPTED_PAGE_SIZE},
    StoreError,
};
#[cfg(test)]
use aura_security::symmetric::generate_key;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::io::{Seek, SeekFrom, Write};
#[cfg(test)]
use tempfile::NamedTempFile;

#[test]
fn test_transparent_encryption() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();

    // Generate a master key
    let master_key = generate_key();

    // Open pager with encryption
    let mut pager = Pager::open(db_path, master_key).unwrap();

    // Create a test page with some data
    let mut page = Page::new(0);
    page.page_type = 1;
    page.used_space = 42;
    page.data[0..4].copy_from_slice(b"test");

    // Write the page (should be encrypted automatically)
    pager.write_page(&page).unwrap();

    // Read the page back (should be decrypted automatically)
    let read_page = pager.read_page(0).unwrap();

    // Verify the data matches
    assert_eq!(read_page.id, page.id);
    assert_eq!(read_page.page_type, page.page_type);
    assert_eq!(read_page.used_space, page.used_space);
    assert_eq!(&read_page.data[0..4], b"test");
}

#[test]
fn test_tamper_detection() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();

    // Generate a master key
    let master_key = generate_key();

    // Open pager with encryption
    let mut pager = Pager::open(db_path, master_key).unwrap();

    // Create and write a test page
    let page = Page::new(0);
    pager.write_page(&page).unwrap();

    // Manually corrupt the encrypted data on disk (corrupt the authentication tag)
    let mut file = fs::OpenOptions::new().write(true).open(db_path).unwrap();
    file.seek(SeekFrom::Start((ENCRYPTED_PAGE_SIZE - 5) as u64))
        .unwrap(); // Seek to near the end of the tag
    file.write_all(b"XXXXX").unwrap();

    // Attempting to read should detect tampering
    let result = pager.read_page(0);
    assert!(matches!(result, Err(StoreError::Tampered(_))));
}

#[test]
fn test_different_keys_produce_different_ciphertext() {
    // Create a temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();

    // Generate different master keys
    let master_key1 = generate_key();
    let master_key2 = generate_key();

    // Create pages with same data
    let mut page = Page::new(0);
    page.data[0..4].copy_from_slice(b"test");

    // Write with key1
    {
        let mut pager1 = Pager::open(db_path, master_key1).unwrap();
        pager1.write_page(&page).unwrap();
    }

    // Read the raw encrypted bytes
    let encrypted_data1 = fs::read(db_path).unwrap();

    // Clear the file and write with key2
    fs::write(db_path, []).unwrap();
    {
        let mut pager2 = Pager::open(db_path, master_key2).unwrap();
        pager2.write_page(&page).unwrap();
    }

    // Read the raw encrypted bytes
    let encrypted_data2 = fs::read(db_path).unwrap();

    // The encrypted data should be different (different keys produce different ciphertext)
    assert_ne!(encrypted_data1, encrypted_data2);

    // Both should be the expected encrypted size
    assert_eq!(encrypted_data1.len(), ENCRYPTED_PAGE_SIZE);
    assert_eq!(encrypted_data2.len(), ENCRYPTED_PAGE_SIZE);

    // The data should not contain the plaintext "test" (appears encrypted)
    assert!(!encrypted_data1.windows(4).any(|w| w == b"test"));
    assert!(!encrypted_data2.windows(4).any(|w| w == b"test"));
}

#[test]
fn test_btree_basic_operations() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();

    // Generate a master key
    let master_key = generate_key();

    // Create pager
    let mut pager = Pager::open(db_path, master_key).unwrap();

    // Create a root node (leaf)
    let root_id = 1;
    let root_node = crate::btree::node::BTreeNode::new_leaf(root_id);

    // Write the root node to disk
    let bytes = root_node.to_bytes().unwrap();
    let mut page = Page::new(root_id);
    page.used_space = bytes.len() as u16;
    page.data[..bytes.len()].copy_from_slice(&bytes);
    pager.write_page(&page).unwrap();

    // Create BTreeManager
    let mut btree = crate::btree::manager::BTreeManager::new(&mut pager, root_id);

    // Test insert
    btree.insert("user_123".to_string(), 42).unwrap();

    // Test search
    let result = btree.search("user_123").unwrap();
    assert_eq!(result, Some(42));

    // Test search for non-existent key
    let result = btree.search("user_999").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_btree_split_and_growth() {
    let file = "test_btree.db";
    let _ = std::fs::remove_file(file);

    let key = generate_key();
    let mut pager = Pager::open(file, key).unwrap();

    // Initialize Tree (Root is Page 1, Page 0 is Primary Index)
    let root_id = pager.allocate_page();
    let root = crate::btree::node::BTreeNode::new_leaf(root_id);

    // Manually write root to start
    let bytes = root.to_bytes().unwrap();
    let mut page = Page::new(root_id);
    page.used_space = bytes.len() as u16;
    page.data[..bytes.len()].copy_from_slice(&bytes);
    pager.write_page(&page).unwrap();

    let mut btree = crate::btree::manager::BTreeManager::new(&mut pager, root_id);

    // INSERT 60 ITEMS (Capacity is 50, so this FORCES a split)
    for i in 0..60 {
        let key = format!("user_{:03}", i); // user_000, user_001...
        btree.insert(key, i + 100).expect("Insert failed");
    }

    // Verify Search works on both sides of the split
    let res1 = btree.search("user_005").unwrap(); // Left side
    let res2 = btree.search("user_055").unwrap(); // Right side

    assert_eq!(res1, Some(105));
    assert_eq!(res2, Some(155));

    println!("✅ B-Tree Successfully Split and Rebalanced!");
}

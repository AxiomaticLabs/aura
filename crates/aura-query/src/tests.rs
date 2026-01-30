#[cfg(test)]
use crate::executor::QueryEngine;
#[cfg(test)]
use aura_common::AuraDocument;
#[cfg(test)]
use aura_security::symmetric;
#[cfg(test)]
use aura_store::pager::Pager;
#[cfg(test)]
use std::fs;

#[test]
fn test_sql_to_encrypted_storage() {
    let db_path = "test_query.db";
    let _ = fs::remove_file(db_path);

    // 1. Setup the Vault
    let key = symmetric::generate_key();
    let mut pager = Pager::open(db_path, key).unwrap();

    // 2. Setup the Brain
    let mut engine = QueryEngine::new(&mut pager);

    // 3. Execute SQL
    let sql = "INSERT INTO users (id, name, age) VALUES ('user_007', 'James', 35)";
    let result = engine.execute(sql).expect("Execution failed");
    println!("✅ Query Result: {}", result);

    // 4. Verify Persistence (Read it back from the Vault)
    // We bypass the QueryEngine and ask the Pager directly to ensure it was written
    let page = pager.read_page(0).expect("Failed to read page 0");

    // Deserialize the data inside the page
    let stored_bytes = &page.data[..page.used_space as usize];
    let doc = AuraDocument::from_bytes(stored_bytes).expect("Failed to deserialize");

    assert_eq!(doc.id, "user_007");
    println!("✅ Data persisted correctly: {:?}", doc);

    // Cleanup
    fs::remove_file(db_path).unwrap();
}

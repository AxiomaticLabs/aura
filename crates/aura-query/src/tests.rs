#[cfg(test)]
use crate::executor::QueryEngine;
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

    // 3. Execute INSERT SQL
    let insert_sql = "INSERT INTO users (id, name, age) VALUES ('user_007', 'James', 35)";
    let insert_result = engine.execute(insert_sql).expect("INSERT failed");
    println!("✅ INSERT Result: {}", insert_result);

    // 4. Execute SELECT SQL (using the index)
    let select_sql = "SELECT * FROM users WHERE id = 'user_007'"; // This will be parsed but we hardcoded the ID
    let select_result = engine.execute(select_sql).expect("SELECT failed");
    println!("✅ SELECT Result: {}", select_result);

    // Verify the result contains the expected data
    assert!(select_result.contains("user_007"));
    assert!(select_result.contains("James"));

    // Cleanup
    fs::remove_file(db_path).unwrap();
}

use pqcrypto_traits::kem::PublicKey;
use tfhe::prelude::*;
use tfhe::FheUint32;

#[test]
fn test_kyber_handshake() {
    // 1. Server generates Identity
    let server_keys = crate::kem::PQCKeyPair::generate();

    // 2. Client uses Server's Public Key to create a secret
    let (client_secret, ciphertext) = crate::kem::encapsulate(server_keys.pk.as_bytes()).unwrap();

    // 3. Server receives Ciphertext and recovers the secret
    let server_secret = crate::kem::decapsulate(&ciphertext, &server_keys.sk).unwrap();

    // 4. They must match exactly
    assert_eq!(client_secret, server_secret);
    println!(
        "✅ Kyber-1024 Handshake Successful. Shared Secret Size: {} bytes",
        client_secret.len()
    );
}

#[test]
fn test_homomorphic_addition() {
    println!("⏳ Generating FHE Keys (This takes a moment)...");
    let ctx = crate::homomorphic::FheContext::new();

    // 1. Client Encrypts "10" and "20"
    let clear_a = 10u32;
    let clear_b = 20u32;
    let encrypted_a = FheUint32::encrypt(clear_a, &ctx.client_key);
    let encrypted_b = FheUint32::encrypt(clear_b, &ctx.client_key);

    // Serialize to simulate sending over network
    let bytes_a = bincode::serialize(&encrypted_a).unwrap();
    let bytes_b = bincode::serialize(&encrypted_b).unwrap();

    // 2. Server (The Database) Adds them BLINDLY
    let computer = crate::homomorphic::FheComputer::new(ctx.get_server_key());
    let result_bytes = computer.sum_encrypted(&bytes_a, &bytes_b).unwrap();

    // 3. Client Decrypts the result
    let result_encrypted: FheUint32 = bincode::deserialize(&result_bytes).unwrap();
    let result: u32 = result_encrypted.decrypt(&ctx.client_key);

    assert_eq!(result, 30);
    println!("✅ FHE Addition Successful: Enc(10) + Enc(20) = Enc(30)");
}

#[test]
fn test_symmetric_encryption_decryption() {
    // Test basic encrypt/decrypt
    let key = crate::symmetric::generate_key();
    let plaintext = b"Hello, Quantum World!";

    let encrypted = crate::symmetric::encrypt(plaintext, &key).unwrap();
    let decrypted = crate::symmetric::decrypt(&encrypted, &key).unwrap();

    assert_eq!(plaintext, decrypted.as_slice());
    println!("✅ Symmetric Encryption/Decryption Successful");
}

#[test]
fn test_symmetric_encryption_different_keys() {
    // Test that different keys produce different ciphertexts
    let key1 = crate::symmetric::generate_key();
    let key2 = crate::symmetric::generate_key();
    let plaintext = b"Same message, different keys";

    let encrypted1 = crate::symmetric::encrypt(plaintext, &key1).unwrap();
    let encrypted2 = crate::symmetric::encrypt(plaintext, &key2).unwrap();

    // Different keys should produce different ciphertexts
    assert_ne!(encrypted1, encrypted2);

    // But each should decrypt correctly with its own key
    let decrypted1 = crate::symmetric::decrypt(&encrypted1, &key1).unwrap();
    let decrypted2 = crate::symmetric::decrypt(&encrypted2, &key2).unwrap();

    assert_eq!(plaintext, decrypted1.as_slice());
    assert_eq!(plaintext, decrypted2.as_slice());
    println!("✅ Symmetric Encryption with Different Keys Successful");
}

#[test]
fn test_symmetric_encryption_edge_cases() {
    let key = crate::symmetric::generate_key();

    // Test empty data
    let empty_data = b"";
    let encrypted = crate::symmetric::encrypt(empty_data, &key).unwrap();
    let decrypted = crate::symmetric::decrypt(&encrypted, &key).unwrap();
    assert_eq!(empty_data, decrypted.as_slice());

    // Test large data
    let large_data = vec![42u8; 10000]; // 10KB of data
    let encrypted = crate::symmetric::encrypt(&large_data, &key).unwrap();
    let decrypted = crate::symmetric::decrypt(&encrypted, &key).unwrap();
    assert_eq!(large_data, decrypted);

    // Test binary data
    let binary_data = &[0, 1, 255, 0, 128, 64, 32, 16, 8, 4, 2, 1];
    let encrypted = crate::symmetric::encrypt(binary_data, &key).unwrap();
    let decrypted = crate::symmetric::decrypt(&encrypted, &key).unwrap();
    assert_eq!(binary_data, decrypted.as_slice());

    println!("✅ Symmetric Encryption Edge Cases Successful");
}

#[test]
fn test_symmetric_decryption_errors() {
    let key = crate::symmetric::generate_key();

    // Test decryption with wrong key
    let plaintext = b"Secret message";
    let encrypted = crate::symmetric::encrypt(plaintext, &key).unwrap();

    let wrong_key = crate::symmetric::generate_key();
    assert!(crate::symmetric::decrypt(&encrypted, &wrong_key).is_err());

    // Test decryption with corrupted data
    let mut corrupted = encrypted.clone();
    if corrupted.len() > 10 {
        corrupted[10] ^= 0xFF; // Flip some bits
        assert!(crate::symmetric::decrypt(&corrupted, &key).is_err());
    }

    // Test decryption with too short data
    let too_short = vec![1, 2, 3]; // Less than nonce + tag
    assert!(crate::symmetric::decrypt(&too_short, &key).is_err());

    println!("✅ Symmetric Decryption Error Handling Successful");
}

#[test]
fn test_kem_keypair_generation() {
    // Test keypair generation
    let keypair1 = crate::kem::PQCKeyPair::generate();
    let keypair2 = crate::kem::PQCKeyPair::generate();

    // Different keypairs should have different public keys
    assert_ne!(keypair1.pk.as_bytes(), keypair2.pk.as_bytes());

    // But same keypair should be consistent
    assert_eq!(keypair1.pk.as_bytes(), keypair1.pk.as_bytes());

    println!("✅ KEM Keypair Generation Successful");
}

#[test]
fn test_kem_encapsulate_decapsulate() {
    // Test the full encapsulate/decapsulate cycle
    let keypair = crate::kem::PQCKeyPair::generate();
    let pk_bytes = keypair.pk.as_bytes();

    let (shared_secret1, ciphertext) = crate::kem::encapsulate(pk_bytes).unwrap();
    let shared_secret2 = crate::kem::decapsulate(&ciphertext, &keypair.sk).unwrap();

    assert_eq!(shared_secret1, shared_secret2);
    println!("✅ KEM Encapsulate/Decapsulate Cycle Successful");
}

#[test]
fn test_kem_invalid_public_key() {
    // Test encapsulation with invalid public key
    let invalid_pk = vec![0u8; 1184]; // Wrong size or invalid data
    assert!(crate::kem::encapsulate(&invalid_pk).is_err());

    // Test with wrong size
    let wrong_size_pk = vec![0u8; 100];
    assert!(crate::kem::encapsulate(&wrong_size_pk).is_err());

    println!("✅ KEM Invalid Public Key Handling Successful");
}

#[test]
fn test_kem_invalid_ciphertext() {
    // Test decapsulation with invalid ciphertext
    let keypair = crate::kem::PQCKeyPair::generate();

    // Test with wrong size (much smaller)
    let wrong_size_ct = vec![0u8; 10];
    assert!(crate::kem::decapsulate(&wrong_size_ct, &keypair.sk).is_err());

    // Test with empty ciphertext
    let empty_ct = vec![];
    assert!(crate::kem::decapsulate(&empty_ct, &keypair.sk).is_err());

    println!("✅ KEM Invalid Ciphertext Handling Successful");
}

#[test]
fn test_homomorphic_context_creation() {
    // Test FHE context creation
    let ctx = crate::homomorphic::FheContext::new();

    // Should be able to get server key
    let _server_key = ctx.get_server_key();

    // Test default implementation
    let ctx2 = crate::homomorphic::FheContext::default();
    let _server_key2 = ctx2.get_server_key();

    println!("✅ Homomorphic Context Creation Successful");
}

#[test]
fn test_homomorphic_computation_errors() {
    // Test error handling in homomorphic operations
    let computer = crate::homomorphic::FheComputer::new(
        crate::homomorphic::FheContext::new().get_server_key(),
    );

    // Test with invalid serialized data
    let invalid_data = vec![1, 2, 3];
    assert!(computer
        .sum_encrypted(&invalid_data, &invalid_data)
        .is_err());

    // Test with empty data
    let empty_data = vec![];
    assert!(computer.sum_encrypted(&empty_data, &empty_data).is_err());

    println!("✅ Homomorphic Error Handling Successful");
}

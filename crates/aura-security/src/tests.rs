#[cfg(test)]
mod tests {
    use super::*;
    use crate::homomorphic::FheContext;
    use tfhe::prelude::*;
    use pqcrypto_traits::kem::PublicKey;

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
        println!("✅ Kyber-1024 Handshake Successful. Shared Secret Size: {} bytes", client_secret.len());
    }

    #[test]
    fn test_homomorphic_addition() {
        println!("⏳ Generating FHE Keys (This takes a moment)...");
        let ctx = FheContext::new();
        
        // 1. Client Encrypts "10" and "20"
        let clear_a = 10u32;
        let clear_b = 20u32;
        let encrypted_a = tfhe::FheUint32::encrypt(clear_a, &ctx.client_key);
        let encrypted_b = tfhe::FheUint32::encrypt(clear_b, &ctx.client_key);

        // Serialize to simulate sending over network
        let bytes_a = bincode::serialize(&encrypted_a).unwrap();
        let bytes_b = bincode::serialize(&encrypted_b).unwrap();

        // 2. Server (The Database) Adds them BLINDLY
        let computer = crate::homomorphic::FheComputer::new(ctx.get_server_key());
        let result_bytes = computer.sum_encrypted(&bytes_a, &bytes_b).unwrap();

        // 3. Client Decrypts the result
        let result_encrypted: tfhe::FheUint32 = bincode::deserialize(&result_bytes).unwrap();
        let result: u32 = result_encrypted.decrypt(&ctx.client_key);

        assert_eq!(result, 30);
        println!("✅ FHE Addition Successful: Enc(10) + Enc(20) = Enc(30)");
    }
}
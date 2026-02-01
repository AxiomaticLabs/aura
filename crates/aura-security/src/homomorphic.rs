use crate::CryptoError;
use tfhe::prelude::*;
use tfhe::{ClientKey, ConfigBuilder, FheUint32, ServerKey};

pub struct FheContext {
    pub client_key: ClientKey, // Held ONLY by the Client
    server_key: ServerKey,     // Held by the Database Engine
}

impl FheContext {
    pub fn new() -> Self {
        let config = ConfigBuilder::default().build();
        let client_key = ClientKey::generate(config);
        let server_key = ServerKey::new(&client_key);
        Self {
            client_key,
            server_key,
        }
    }

    pub fn get_server_key(&self) -> ServerKey {
        self.server_key.clone()
    }
}

/// Operations the Database can perform BLINDLY
pub struct FheComputer {
    server_key: ServerKey,
}

impl FheComputer {
    pub fn new(server_key: ServerKey) -> Self {
        Self { server_key }
    }

    /// Adds two encrypted integers without decrypting them
    pub fn sum_encrypted(&self, a_bytes: &[u8], b_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // 1. Deserialize the encrypted blobs
        let a: FheUint32 =
            bincode::deserialize(a_bytes).map_err(|_| CryptoError::DecryptionFailed)?;
        let b: FheUint32 =
            bincode::deserialize(b_bytes).map_err(|_| CryptoError::DecryptionFailed)?;

        // 2. Perform the Math (CPU Intensive!)
        // Notice: We define the server_key context to perform the operation
        tfhe::set_server_key(self.server_key.clone());
        let result = &a + &b;

        // 3. Serialize the encrypted result back to bytes
        bincode::serialize(&result).map_err(|_| CryptoError::DecryptionFailed)
    }
}

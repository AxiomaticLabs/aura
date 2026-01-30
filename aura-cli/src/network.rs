use anyhow::{Context, Result};
use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SharedSecret};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct AuraClient {
    stream: TcpStream,
    #[allow(dead_code)] // We will use this in Step 9 for encryption
    session_key: Vec<u8>,
}

impl AuraClient {
    /// Connects to the server and performs the PQC Handshake
    pub async fn connect(addr: &str) -> Result<Self> {
        println!("🔌 Connecting to {}...", addr);
        let mut stream = TcpStream::connect(addr)
            .await
            .context("Failed to connect to AuraDB Server")?;

        // --- STEP 1: HANDSHAKE (The Quantum Shield) ---

        // A. Receive Server's Public Key
        // Kyber1024 Public Key is 1568 bytes
        let mut pk_buffer = vec![0u8; 1568];
        stream
            .read_exact(&mut pk_buffer)
            .await
            .context("Failed to receive Server Public Key")?;

        // B. Encapsulate (Create Shared Secret)
        let pk =
            kyber1024::PublicKey::from_bytes(&pk_buffer).context("Invalid public key received")?;
        let (shared_secret, ciphertext) = kyber1024::encapsulate(&pk);

        // C. Send Ciphertext to Server
        stream
            .write_all(ciphertext.as_bytes())
            .await
            .context("Failed to send Ciphertext")?;

        println!("🔒 Handshake Complete. Quantum Secure Session Established.");

        Ok(Self {
            stream,
            session_key: shared_secret.as_bytes().to_vec(),
        })
    }

    /// Sends a raw SQL query and gets a response
    pub async fn send_query(&mut self, query: &str) -> Result<String> {
        // --- STEP 2: TRANSPORT ---

        // For Step 8, we send the query text directly to verify the pipe works.
        // In Step 9, we will wrap this with `symmetric::encrypt(payload, &self.session_key)`
        let payload = query.as_bytes();

        // Write Payload
        self.stream.write_all(payload).await?;

        // Read Response
        let mut buffer = [0u8; 4096];
        let n = self.stream.read(&mut buffer).await?;

        let response = String::from_utf8_lossy(&buffer[0..n]).to_string();
        Ok(response)
    }
}

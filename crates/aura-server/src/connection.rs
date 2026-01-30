use anyhow::{bail, Result};
use aura_query::executor::QueryEngine;
use aura_security::kem;
use aura_store::pager::Pager;
use pqcrypto_traits::kem::PublicKey;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, info};

// The Protocol States
// TODO: Implement encrypted communication using session_key
#[allow(dead_code)]
enum ConnectionState {
    Handshake,
    Authenticated { session_key: Vec<u8> },
}

pub async fn handle_socket(mut socket: TcpStream, db: Arc<Mutex<Pager>>) -> Result<()> {
    let mut state = ConnectionState::Handshake;
    let mut buffer = [0u8; 4096];

    loop {
        match state {
            // --- STEP 1: QUANTUM HANDSHAKE ---
            ConnectionState::Handshake => {
                debug!("Initiating PQC Handshake...");

                // A. Server generates ephemeral Kyber Keypair
                let server_keys = kem::PQCKeyPair::generate();

                // B. Send Public Key to Client (1184 bytes)
                socket.write_all(server_keys.pk.as_bytes()).await?;

                // C. Wait for Client's Encapsulated Secret
                let n = socket.read(&mut buffer).await?;
                if n == 0 {
                    return Ok(());
                } // Client disconnected

                // D. Decapsulate to get Shared Secret
                // In production, frame this properly. Kyber1024 Ciphertext is 1568 bytes.
                let ciphertext = &buffer[0..n];

                let shared_secret = match kem::decapsulate(ciphertext, &server_keys.sk) {
                    Ok(secret) => secret,
                    Err(_) => {
                        bail!("Handshake Failed: Invalid Kyber Ciphertext");
                    }
                };

                // E. Upgrade State
                state = ConnectionState::Authenticated {
                    session_key: shared_secret,
                };
                info!("🔒 Handshake Success. Secure Channel Established.");
            }

            // --- STEP 2: SECURE COMMAND LOOP ---
            ConnectionState::Authenticated { session_key: _ } => {
                // A. Read Encrypted Request
                let n = socket.read(&mut buffer).await?;
                if n == 0 {
                    return Ok(());
                }

                let encrypted_req = &buffer[0..n];

                // B. Decrypt (Using the Shared Session Key)
                // For Step 7, we skip actual decryption to test connectivity first.
                // TODO: Wrap this in symmetric::decrypt(encrypted_req, session_key)
                let request_str = String::from_utf8_lossy(encrypted_req).trim().to_string();
                debug!("Received Query: {}", request_str);

                // C. Execute Query
                let response = {
                    // Lock the DB, Execute, Unlock immediately
                    let mut engine_lock = db.lock().await;
                    let mut query_engine = QueryEngine::new(&mut *engine_lock);

                    match query_engine.execute(&request_str) {
                        Ok(res) => format!("OK: {}", res),
                        Err(e) => format!("ERROR: {}", e),
                    }
                };

                // D. Send Response (Should be Encrypted)
                // TODO: Wrap in symmetric::encrypt(response, session_key)
                socket.write_all(response.as_bytes()).await?;
            }
        }
    }
}

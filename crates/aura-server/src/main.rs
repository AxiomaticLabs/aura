mod connection;
mod protocol;

use aura_security::symmetric;
use aura_store::pager::Pager;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    tracing_subscriber::fmt::init();
    info!(
        "🚀 AuraDB 'Unbreakable' Server Starting... (Protocol v{})",
        protocol::PROTOCOL_VERSION
    );

    // 2. Initialize The Vault (Thread-Safe)
    // In production, we'd load this key from a secure KMS.
    // For now, we generate a new one on startup (Ephemeral DB).
    // To persist, save this key to a file!
    info!("🔑 Generating Master Key (Memory Only)...");
    let master_key = symmetric::generate_key();

    // Open the DB file
    let pager =
        Pager::open("aura_main.db", master_key).expect("Failed to initialize storage engine");

    // Wrap in Arc<Mutex> so multiple TCP threads can access it safely
    let db_engine = Arc::new(Mutex::new(pager));

    // 3. Start TCP Listener
    let addr = "0.0.0.0:7654"; // Port 7654 (PQL - Post Quantum Link)
    let listener = TcpListener::bind(addr).await?;
    info!("✅ Listening on {} for Secure Connections", addr);

    loop {
        // 4. Accept Incoming Connection
        let (socket, remote_addr) = listener.accept().await?;
        info!("🔗 New connection from {}", remote_addr);

        let db_ref = db_engine.clone();

        // 5. Spawn a dedicated async task for this client
        tokio::spawn(async move {
            if let Err(e) = connection::handle_socket(socket, db_ref).await {
                error!("❌ Connection Error [{}]: {}", remote_addr, e);
            }
        });
    }
}

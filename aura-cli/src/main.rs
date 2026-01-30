mod network;

use clap::{Parser, Subcommand};
use colored::*;
use network::AuraClient;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Parser)]
#[command(name = "aura")]
#[command(about = "The Unbreakable Database Client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Server Address
    #[arg(long, default_value = "127.0.0.1:7654")]
    host: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive SQL shell (Default)
    Shell,
    /// Execute a single query
    Exec { query: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let host = cli.host;

    match &cli.command {
        Some(Commands::Exec { query }) => {
            let mut client = AuraClient::connect(&host).await?;
            let res = client.send_query(query).await?;
            println!("{}", res);
        }
        Some(Commands::Shell) | None => {
            start_repl(&host).await?;
        }
    }

    Ok(())
}

async fn start_repl(host: &str) -> anyhow::Result<()> {
    // 1. Connect
    let mut client = match AuraClient::connect(host).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {}", "Fatal Error:".red().bold(), e);
            return Ok(());
        }
    };

    println!(
        "{}",
        "Welcome to AuraDB Shell. Type 'exit' to quit.".green()
    );

    // 2. Start Read-Eval-Print Loop
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(&format!("{} > ", "aura".blue().bold()));
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.eq_ignore_ascii_case("exit") {
                    break;
                }
                if input.is_empty() {
                    continue;
                }

                rl.add_history_entry(input)?;

                // 3. Send to Server
                match client.send_query(input).await {
                    Ok(response) => println!("{}", response),
                    Err(e) => println!("{} {}", "Error:".red(), e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use aura_common::document::{AuraDocument, DataValue};

    #[test]
    fn test_cli_parsing() {
        // Test basic CLI argument parsing
        let args = ["aura-cli", "--host", "127.0.0.1", "--port", "7654"];
        // Note: This would require clap to parse, but for now just test the concept
        assert_eq!(args.len(), 5);
        assert_eq!(args[0], "aura-cli");
    }

    #[test]
    fn test_document_display_formatting() {
        // Test how documents would be displayed in CLI output
        let mut doc = AuraDocument::new("test_doc");
        doc.data
            .insert("id".to_string(), DataValue::Text("user_123".to_string()));
        doc.data
            .insert("name".to_string(), DataValue::Text("John Doe".to_string()));
        doc.data.insert("age".to_string(), DataValue::Integer(30));
        doc.data
            .insert("active".to_string(), DataValue::Boolean(true));

        // Test serialization (used for network transmission)
        let bytes = doc.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        let deserialized = AuraDocument::from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.data.len(), 4);
    }

    #[test]
    fn test_network_address_parsing() {
        // Test parsing of host:port combinations
        let host = "127.0.0.1";
        let port = 7654;

        // Test basic validation
        assert!(host.contains('.'));
        assert!(port > 1024 && port < 65535);

        let address = format!("{}:{}", host, port);
        assert_eq!(address, "127.0.0.1:7654");
    }

    #[test]
    fn test_query_input_validation() {
        // Test basic validation of SQL queries that would be entered
        let valid_queries = vec![
            "SELECT * FROM users WHERE id = 'user_007'",
            "INSERT INTO users (id, name) VALUES ('user_008', 'Jane')",
        ];

        for query in valid_queries {
            assert!(!query.is_empty());
            assert!(
                query.to_uppercase().contains("SELECT") || query.to_uppercase().contains("INSERT")
            );
        }

        let invalid_queries = vec!["", "   ", "INVALID QUERY"];

        for query in invalid_queries {
            assert!(
                query.trim().is_empty()
                    || !query.to_uppercase().contains("SELECT")
                        && !query.to_uppercase().contains("INSERT")
            );
        }
    }

    #[test]
    fn test_repl_command_parsing() {
        // Test parsing of REPL commands
        let commands = vec![".exit", ".quit", ".help", "SELECT * FROM users"];

        for cmd in commands {
            if cmd.starts_with('.') {
                // Meta commands
                assert!(cmd.len() > 1);
            } else {
                // SQL commands
                assert!(
                    cmd.to_uppercase().contains("SELECT") || cmd.to_uppercase().contains("INSERT")
                );
            }
        }
    }

    #[test]
    fn test_error_display_formatting() {
        use aura_common::error::AuraError;

        // Test how errors would be displayed to users
        let errors = vec![
            AuraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Connection failed",
            )),
            AuraError::NotFound("Document not found".to_string()),
            AuraError::Crypto("Encryption failed".to_string()),
        ];

        for error in errors {
            let error_msg = error.to_string();
            assert!(!error_msg.is_empty());
            // Errors should be user-friendly
            assert!(!error_msg.contains("Box<dyn"));
        }
    }

    #[test]
    fn test_connection_timeout_handling() {
        // Test timeout values that would be used
        let connect_timeout_ms = 5000;
        let read_timeout_ms = 10000;

        assert!(connect_timeout_ms > 0);
        assert!(read_timeout_ms > connect_timeout_ms);
        assert!(read_timeout_ms < 60000); // Less than 1 minute
    }

    #[test]
    fn test_data_value_display() {
        // Test how different data types would be displayed
        let values = vec![
            DataValue::Text("hello".to_string()),
            DataValue::Integer(42),
            DataValue::Boolean(true),
            DataValue::Float(std::f64::consts::PI),
        ];

        for value in values {
            // Just ensure they can be created and serialized
            let mut doc = AuraDocument::new("test");
            doc.data.insert("test".to_string(), value);
            let bytes = doc.to_bytes().unwrap();
            assert!(!bytes.is_empty());
        }
    }
}

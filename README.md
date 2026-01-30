# AuraDB

[![CI](https://github.com/AxiomaticLabs/aura/actions/workflows/ci.yml/badge.svg)](https://github.com/AxiomaticLabs/aura/actions/workflows/ci.yml)

A next-generation database combining SQL and NoSQL capabilities with post-quantum cryptography and homomorphic encryption.

## Features

- **Hybrid Data Model**: Supports both SQL rows and NoSQL documents in a single engine
- **Post-Quantum Security**: Uses Kyber-1024 for key exchange and Dilithium for signatures
- **Homomorphic Encryption**: Perform computations on encrypted data without decryption
- **Multi-Version Concurrency Control (MVCC)**: Advanced concurrency control
- **Cross-Platform**: Runs on Linux, macOS, and Windows

## Architecture

- `aura-common`: Core data structures and serialization
- `aura-security`: Cryptographic primitives and FHE operations
- `aura-store`: Storage engine with encryption
- `aura-query`: Query processing and optimization
- `aura-server`: Network server and API
- `aura-cli`: Command-line interface

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test --workspace
```

## Installation & Packaging

AuraDB provides professional system packages for easy deployment as a background service.

### Quick Install (Development)

```bash
# Build and run locally
cargo build --release
./target/release/aura-server &
./target/release/aura-cli
```

### Production Installation

#### Linux (Ubuntu/Debian/Fedora)

**Using .deb packages:**
```bash
# Install cargo-deb
cargo install cargo-deb

# Build packages
cargo deb -p aura-server
cargo deb -p aura-cli

# Install (creates systemd service automatically)
sudo dpkg -i target/debian/aura-server_*.deb
sudo dpkg -i target/debian/aura-cli_*.deb

# Start service
sudo systemctl start aura-server
sudo systemctl enable aura-server
```

**Using .rpm packages (Fedora/RHEL):**
```bash
# Install cargo-generate-rpm
cargo install cargo-generate-rpm

# Build RPMs
cargo generate-rpm -p aura-server
cargo generate-rpm -p aura-cli

# Install
sudo rpm -i target/generate-rpm/aura-server-*.rpm
sudo rpm -i target/generate-rpm/aura-cli-*.rpm
```

#### Windows

**Using .msi installers:**
```bash
# Install cargo-wix
cargo install cargo-wix

# Build MSI installers
cargo wix -p aura-server
cargo wix -p aura-cli

# Install (double-click the .msi files)
# This creates a Windows Service and adds binaries to PATH
```

#### macOS

**Using .pkg installer:**
```bash
# Run the macOS build script
./build_mac_pkg.sh

# Install
sudo installer -pkg target/aura-db-*.pkg -target /

# Start service
sudo launchctl load /Library/LaunchDaemons/com.aura.db.plist
```

### Universal Build Script

Build packages for all platforms:

```bash
# Build for current platform
./build_packages.sh

# Build for specific platform
./build_packages.sh linux    # .deb/.rpm
./build_packages.sh windows  # .msi
./build_packages.sh macos    # .pkg

# Build for all platforms (if cross-compiling setup)
./build_packages.sh all
```

### Service Management

**Linux (systemd):**
```bash
sudo systemctl status aura-server
sudo systemctl restart aura-server
sudo systemctl stop aura-server
journalctl -u aura-server -f  # View logs
```

**Windows (services.msc):**
- Open Services.msc
- Find "AuraDB" service
- Start/Stop/Restart as needed

**macOS (launchd):**
```bash
sudo launchctl list com.aura.db
sudo launchctl start com.aura.db
sudo launchctl stop com.aura.db
tail -f /var/log/aura.log  # View logs
```

### Directory Structure

After installation:
- **Binaries:** `/usr/bin/aura-server`, `/usr/bin/aura`
- **Data:** `/var/lib/aura/` (Linux/macOS), `C:\ProgramData\aura\` (Windows)
- **Logs:** `/var/log/aura/` (Linux/macOS), Event Viewer (Windows)
- **Config:** System service configuration files

### Security Features

- **System User:** Runs as dedicated `aura` user (Linux/macOS) or `LocalSystem` (Windows)
- **Minimal Permissions:** No shell access, restricted file permissions
- **Automatic Updates:** Service restarts on failure
- **Encrypted Storage:** All data encrypted at rest

## License

[License information here]
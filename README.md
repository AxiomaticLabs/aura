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

## License

[License information here]
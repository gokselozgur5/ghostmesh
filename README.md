# ghostmesh

Serverless P2P Encrypted Mesh & Local-First CRDT Sync Engine in Rust.

## Architecture

- `ghostmesh-core`: Cryptographic identity (Ed25519), E2EE session framing (X25519 + BLAKE3 KDF + ChaCha20-Poly1305), HLC-ordered CRDTs (LWW-Map, OR-Set), and BLAKE3 chunked transfer.
- `ghostmesh-cli`: Diagnostic CLI utility suite for key management, E2EE sync simulation, and chunk transfer integrity checks.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

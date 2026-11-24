# ghostmesh

Serverless P2P Encrypted Mesh & Local-First CRDT Sync Engine in Rust.

## Architecture

- `ghostmesh-core`: Cryptographic identity (Ed25519), E2EE session framing (X25519 + BLAKE3 KDF + ChaCha20-Poly1305), HLC-ordered CRDTs (LWW-Map, OR-Set), and BLAKE3 chunked transfer.
- `ghostmesh-net`: Live P2P encrypted transport, length-prefixed framing, ephemeral X25519+Ed25519 handshake, UDP LAN beacon discovery, and multi-node CRDT sync & file transfer runtime.
- `ghostmesh-cli`: Diagnostic CLI utility suite for key management, E2EE sync simulation, live multi-node mesh demos, listener endpoints, and chunk transfer integrity checks.

## CLI Commands

```bash
ghostmesh keygen                       # Generate new Ed25519 peer identity
ghostmesh sim-sync                     # Simulate encrypted E2EE CRDT sync session
ghostmesh pack <file>                  # Create BLAKE3 chunked manifest for file
ghostmesh verify <manifest> <source>   # Verify file integrity against manifest
ghostmesh live-demo                    # Run live encrypted 2-node CRDT + file transfer demo
ghostmesh listen <addr>                # Listen as a live mesh node on address
ghostmesh send <addr> <file>           # Send file to remote mesh node over encrypted link
```

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

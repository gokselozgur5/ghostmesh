use ed25519_dalek::{
    Signature, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};
use ghostmesh_core::identity::{IdentityError, PeerId, PeerIdentity};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use x25519_dalek::PublicKey as X25519PublicKey;

pub const INITIATOR_ROLE: &[u8] = b"ghostmesh-handshake-initiator-v1";
pub const RESPONDER_ROLE: &[u8] = b"ghostmesh-handshake-responder-v1";

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("wire error: {0}")]
    Wire(#[from] crate::wire::WireError),
    #[error("identity error: {0}")]
    Identity(#[from] IdentityError),
    #[error("crypto error: {0}")]
    Crypto(#[from] ghostmesh_core::CryptoError),
    #[error("peer id mismatch with verifying key")]
    PeerIdMismatch,
    #[error("signature verification failed")]
    InvalidSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeHello {
    pub peer_id: PeerId,
    pub verifying_key_bytes: [u8; PUBLIC_KEY_LENGTH],
    pub ephemeral_public: [u8; 32],
    pub signature: Vec<u8>,
}

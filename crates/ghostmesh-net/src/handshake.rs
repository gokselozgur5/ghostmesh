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

impl HandshakeHello {
    pub fn new(
        identity: &PeerIdentity,
        ephemeral_public: &X25519PublicKey,
        role: &[u8],
    ) -> Self {
        let mut msg = Vec::with_capacity(64);
        msg.extend_from_slice(ephemeral_public.as_bytes());
        msg.extend_from_slice(role);
        let sig = identity.sign(&msg);

        Self {
            peer_id: identity.peer_id(),
            verifying_key_bytes: identity.verifying_key().to_bytes(),
            ephemeral_public: *ephemeral_public.as_bytes(),
            signature: sig.to_vec(),
        }
    }

    pub fn verify(&self, expected_role: &[u8]) -> Result<X25519PublicKey, HandshakeError> {
        let vk = VerifyingKey::from_bytes(&self.verifying_key_bytes)
            .map_err(|_| HandshakeError::InvalidSignature)?;
        if self.peer_id != PeerId::from_verifying_key(&vk) {
            return Err(HandshakeError::PeerIdMismatch);
        }
        let mut msg = Vec::with_capacity(64);
        msg.extend_from_slice(&self.ephemeral_public);
        msg.extend_from_slice(expected_role);

        if self.signature.len() != SIGNATURE_LENGTH {
            return Err(HandshakeError::InvalidSignature);
        }
        let sig_arr: [u8; SIGNATURE_LENGTH] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| HandshakeError::InvalidSignature)?;
        let sig = Signature::from_bytes(&sig_arr);
        vk.verify(&msg, &sig)
            .map_err(|_| HandshakeError::InvalidSignature)?;

        Ok(X25519PublicKey::from(self.ephemeral_public))
    }
}

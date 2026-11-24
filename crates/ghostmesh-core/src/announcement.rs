use crate::identity::{IdentityError, PeerId, PeerIdentity};
use ed25519_dalek::{Signature, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerAnnouncement {
    pub peer_id: PeerId,
    pub verifying_key_bytes: [u8; PUBLIC_KEY_LENGTH],
    pub timestamp_ms: u64,
    pub signature: Vec<u8>,
}

impl PeerAnnouncement {
    pub fn new(identity: &PeerIdentity, timestamp_ms: u64) -> Self {
        let mut payload = Vec::with_capacity(72);
        payload.extend_from_slice(&identity.peer_id().0);
        payload.extend_from_slice(identity.verifying_key().as_bytes());
        payload.extend_from_slice(&timestamp_ms.to_be_bytes());

        let sig = identity.sign(&payload);
        Self {
            peer_id: identity.peer_id(),
            verifying_key_bytes: identity.verifying_key().to_bytes(),
            timestamp_ms,
            signature: sig.to_vec(),
        }
    }

    pub fn verify(&self) -> Result<(), IdentityError> {
        let vk = VerifyingKey::from_bytes(&self.verifying_key_bytes)
            .map_err(|_| IdentityError::InvalidLength)?;
        if self.peer_id != PeerId::from_verifying_key(&vk) {
            return Err(IdentityError::SignatureVerificationFailed);
        }

        let mut payload = Vec::with_capacity(72);
        payload.extend_from_slice(&self.peer_id.0);
        payload.extend_from_slice(&self.verifying_key_bytes);
        payload.extend_from_slice(&self.timestamp_ms.to_be_bytes());

        if self.signature.len() != SIGNATURE_LENGTH {
            return Err(IdentityError::InvalidLength);
        }
        let sig_arr: [u8; SIGNATURE_LENGTH] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| IdentityError::InvalidLength)?;
        let sig = Signature::from_bytes(&sig_arr);
        vk.verify(&payload, &sig)
            .map_err(|_| IdentityError::SignatureVerificationFailed)
    }
}

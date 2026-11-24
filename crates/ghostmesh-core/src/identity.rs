use ed25519_dalek::{Signer, SigningKey, VerifyingKey, SIGNATURE_LENGTH};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentityError {
    #[error("invalid byte length for key or signature")]
    InvalidLength,
    #[error("signature verification failed")]
    SignatureVerificationFailed,
    #[error("hex decoding error: {0}")]
    HexDecode(String),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    pub fn from_verifying_key(vk: &VerifyingKey) -> Self {
        let hash = blake3::hash(vk.as_bytes());
        Self(*hash.as_bytes())
    }

    pub fn to_hex(self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn from_hex(s: &str) -> Result<Self, IdentityError> {
        let s = s.trim();
        if s.len() != 64 {
            return Err(IdentityError::InvalidLength);
        }
        let mut bytes = [0u8; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            let chunk = &s[i * 2..i * 2 + 2];
            *byte = u8::from_str_radix(chunk, 16)
                .map_err(|e| IdentityError::HexDecode(e.to_string()))?;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({})", self.to_hex())
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

pub struct PeerIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    peer_id: PeerId,
}

impl PeerIdentity {
    pub fn generate<R: RngCore + CryptoRng>(csprng: &mut R) -> Self {
        let signing_key = SigningKey::generate(csprng);
        let verifying_key = signing_key.verifying_key();
        let peer_id = PeerId::from_verifying_key(&verifying_key);
        Self {
            signing_key,
            verifying_key,
            peer_id,
        }
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        let peer_id = PeerId::from_verifying_key(&verifying_key);
        Self {
            signing_key,
            verifying_key,
            peer_id,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    pub fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_LENGTH] {
        self.signing_key.sign(message).to_bytes()
    }
}

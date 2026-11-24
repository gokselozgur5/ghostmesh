use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use zeroize::{Zeroize, ZeroizeOnDrop};

const KDF_DOMAIN: &[u8] = b"ghostmesh-v1-x25519-chacha20poly1305";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("aead encryption error")]
    EncryptionFailed,
    #[error("aead decryption or authentication failed")]
    DecryptionFailed,
    #[error("nonce counter exhausted (maximum 2^64-1 reached)")]
    NonceExhausted,
    #[error("received nonce {0} is out of order or replayed, expected {1}")]
    ReplayOrOutOfOrder(u64, u64),
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKey(pub [u8; 32]);

pub struct KeyExchange {
    secret: EphemeralSecret,
    public: X25519PublicKey,
}

impl KeyExchange {
    pub fn new<R: rand_core::RngCore + rand_core::CryptoRng>(csprng: &mut R) -> Self {
        let secret = EphemeralSecret::random_from_rng(csprng);
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
    }

    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public
    }

    pub fn derive_session_keys(
        self,
        peer_public: &X25519PublicKey,
        is_initiator: bool,
    ) -> (SessionCipher, SessionCipher) {
        let shared_secret = self.secret.diffie_hellman(peer_public);

        let mut initiator_hasher = blake3::Hasher::new_derive_key(
            "ghostmesh-kdf-session-keys-initiator-to-responder-v1",
        );
        initiator_hasher.update(KDF_DOMAIN);
        initiator_hasher.update(shared_secret.as_bytes());
        let init_to_resp = *initiator_hasher.finalize().as_bytes();

        let mut responder_hasher = blake3::Hasher::new_derive_key(
            "ghostmesh-kdf-session-keys-responder-to-initiator-v1",
        );
        responder_hasher.update(KDF_DOMAIN);
        responder_hasher.update(shared_secret.as_bytes());
        let resp_to_init = *responder_hasher.finalize().as_bytes();

        if is_initiator {
            (
                SessionCipher::new(SessionKey(init_to_resp)),
                SessionCipher::new(SessionKey(resp_to_init)),
            )
        } else {
            (
                SessionCipher::new(SessionKey(resp_to_init)),
                SessionCipher::new(SessionKey(init_to_resp)),
            )
        }
    }
}

pub struct SessionCipher {
    key: SessionKey,
    next_nonce: u64,
}

impl SessionCipher {
    pub fn new(key: SessionKey) -> Self {
        Self { key, next_nonce: 0 }
    }

    pub fn next_nonce(&self) -> u64 {
        self.next_nonce
    }

    fn make_nonce(counter: u64) -> Nonce {
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..12].copy_from_slice(&counter.to_be_bytes());
        *Nonce::from_slice(&nonce_bytes)
    }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<(u64, Vec<u8>), CryptoError> {
        if self.next_nonce == u64::MAX {
            return Err(CryptoError::NonceExhausted);
        }
        let counter = self.next_nonce;
        self.next_nonce += 1;

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key.0)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        let nonce = Self::make_nonce(counter);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        Ok((counter, ciphertext))
    }

    pub fn decrypt(&mut self, counter: u64, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if counter != self.next_nonce {
            return Err(CryptoError::ReplayOrOutOfOrder(counter, self.next_nonce));
        }

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key.0)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        let nonce = Self::make_nonce(counter);
        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        self.next_nonce += 1;
        Ok(plaintext)
    }
}

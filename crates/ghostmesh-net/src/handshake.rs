use ed25519_dalek::{Signature, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
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
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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
    pub fn new(identity: &PeerIdentity, ephemeral_public: &X25519PublicKey, role: &[u8]) -> Self {
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

    pub fn new_server(
        identity: &PeerIdentity,
        ephemeral_public: &X25519PublicKey,
        transcript: &[u8],
    ) -> Self {
        let mut msg = Vec::with_capacity(transcript.len() + RESPONDER_ROLE.len());
        msg.extend_from_slice(transcript);
        msg.extend_from_slice(RESPONDER_ROLE);
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

    pub fn verify_server(&self, transcript: &[u8]) -> Result<X25519PublicKey, HandshakeError> {
        let vk = VerifyingKey::from_bytes(&self.verifying_key_bytes)
            .map_err(|_| HandshakeError::InvalidSignature)?;
        if self.peer_id != PeerId::from_verifying_key(&vk) {
            return Err(HandshakeError::PeerIdMismatch);
        }
        let mut msg = Vec::with_capacity(transcript.len() + RESPONDER_ROLE.len());
        msg.extend_from_slice(transcript);
        msg.extend_from_slice(RESPONDER_ROLE);

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

use ghostmesh_core::{KeyExchange, SessionCipher};
use std::net::TcpStream;

pub struct EncryptedStream {
    pub stream: TcpStream,
    pub tx_cipher: SessionCipher,
    pub rx_cipher: SessionCipher,
    pub remote_peer: PeerId,
}

impl EncryptedStream {
    pub fn remote_peer(&self) -> PeerId {
        self.remote_peer
    }

    pub fn send_encrypted(&mut self, payload: &[u8]) -> Result<(), HandshakeError> {
        let (nonce, ciphertext) = self.tx_cipher.encrypt(payload)?;
        let mut framed = Vec::with_capacity(8 + ciphertext.len());
        framed.extend_from_slice(&nonce.to_be_bytes());
        framed.extend_from_slice(&ciphertext);
        crate::wire::write_frame(&mut self.stream, &framed)?;
        Ok(())
    }

    pub fn recv_encrypted(&mut self) -> Result<Vec<u8>, HandshakeError> {
        let framed = crate::wire::read_frame(&mut self.stream)?;
        if framed.len() < 8 {
            return Err(HandshakeError::Crypto(
                ghostmesh_core::CryptoError::DecryptionFailed,
            ));
        }
        let nonce = u64::from_be_bytes(framed[0..8].try_into().unwrap());
        let ciphertext = &framed[8..];
        let plaintext = self.rx_cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn client_handshake<R: rand_core::RngCore + rand_core::CryptoRng>(
    mut stream: TcpStream,
    identity: &PeerIdentity,
    csprng: &mut R,
) -> Result<EncryptedStream, HandshakeError> {
    let kx = KeyExchange::new(csprng);
    let my_hello = HandshakeHello::new(identity, kx.public_key(), INITIATOR_ROLE);
    let my_bytes = serde_json::to_vec(&my_hello)?;
    crate::wire::write_frame(&mut stream, &my_bytes)?;

    let peer_bytes = crate::wire::read_frame(&mut stream)?;
    let peer_hello: HandshakeHello = serde_json::from_slice(&peer_bytes)?;

    let mut transcript = Vec::with_capacity(160);
    transcript.extend_from_slice(&identity.peer_id().0);
    transcript.extend_from_slice(identity.verifying_key().as_bytes());
    transcript.extend_from_slice(kx.public_key().as_bytes());
    transcript.extend_from_slice(&peer_hello.peer_id.0);
    transcript.extend_from_slice(&peer_hello.verifying_key_bytes);
    transcript.extend_from_slice(&peer_hello.ephemeral_public);

    let peer_kx_pub = peer_hello.verify_server(&transcript)?;

    let (mut tx_cipher, rx_cipher) =
        kx.derive_session_keys_with_transcript(&peer_kx_pub, true, &transcript)?;

    let (nonce, ct) = tx_cipher.encrypt(b"ghostmesh-handshake-finish-v1")?;
    let mut finish_frame = Vec::with_capacity(8 + ct.len());
    finish_frame.extend_from_slice(&nonce.to_be_bytes());
    finish_frame.extend_from_slice(&ct);
    crate::wire::write_frame(&mut stream, &finish_frame)?;

    Ok(EncryptedStream {
        stream,
        tx_cipher,
        rx_cipher,
        remote_peer: peer_hello.peer_id,
    })
}

pub fn server_handshake<R: rand_core::RngCore + rand_core::CryptoRng>(
    mut stream: TcpStream,
    identity: &PeerIdentity,
    csprng: &mut R,
) -> Result<EncryptedStream, HandshakeError> {
    let peer_bytes = crate::wire::read_frame(&mut stream)?;
    let peer_hello: HandshakeHello = serde_json::from_slice(&peer_bytes)?;
    let peer_kx_pub = peer_hello.verify(INITIATOR_ROLE)?;

    let kx = KeyExchange::new(csprng);
    let mut transcript = Vec::with_capacity(160);
    transcript.extend_from_slice(&peer_hello.peer_id.0);
    transcript.extend_from_slice(&peer_hello.verifying_key_bytes);
    transcript.extend_from_slice(&peer_hello.ephemeral_public);
    transcript.extend_from_slice(&identity.peer_id().0);
    transcript.extend_from_slice(identity.verifying_key().as_bytes());
    transcript.extend_from_slice(kx.public_key().as_bytes());

    let my_hello = HandshakeHello::new_server(identity, kx.public_key(), &transcript);
    let my_bytes = serde_json::to_vec(&my_hello)?;
    crate::wire::write_frame(&mut stream, &my_bytes)?;

    let (tx_cipher, mut rx_cipher) =
        kx.derive_session_keys_with_transcript(&peer_kx_pub, false, &transcript)?;

    let confirm_frame = crate::wire::read_frame(&mut stream)?;
    if confirm_frame.len() < 8 {
        return Err(HandshakeError::Crypto(ghostmesh_core::CryptoError::DecryptionFailed));
    }
    let nonce = u64::from_be_bytes(confirm_frame[0..8].try_into().unwrap());
    let plaintext = rx_cipher.decrypt(nonce, &confirm_frame[8..])?;
    if plaintext != b"ghostmesh-handshake-finish-v1" {
        return Err(HandshakeError::InvalidSignature);
    }

    Ok(EncryptedStream {
        stream,
        tx_cipher,
        rx_cipher,
        remote_peer: peer_hello.peer_id,
    })
}

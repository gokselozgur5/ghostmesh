use ghostmesh_core::identity::{IdentityError, PeerIdentity};
use ghostmesh_core::PeerAnnouncement;
use serde::{Deserialize, Serialize};
use std::io;
use std::net::UdpSocket;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("identity error: {0}")]
    Identity(#[from] IdentityError),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryBeacon {
    pub announcement: PeerAnnouncement,
    pub tcp_port: u16,
    pub port_signature: Vec<u8>,
}

impl DiscoveryBeacon {
    pub fn new(identity: &PeerIdentity, tcp_port: u16, timestamp_ms: u64) -> Self {
        let announcement = PeerAnnouncement::new(identity, timestamp_ms);
        let mut payload = Vec::with_capacity(80);
        payload.extend_from_slice(&announcement.peer_id.0);
        payload.extend_from_slice(&announcement.verifying_key_bytes);
        payload.extend_from_slice(&announcement.timestamp_ms.to_be_bytes());
        payload.extend_from_slice(&tcp_port.to_be_bytes());
        payload.extend_from_slice(b"ghostmesh-beacon-v1");
        let port_signature = identity.sign(&payload).to_vec();

        Self {
            announcement,
            tcp_port,
            port_signature,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, DiscoveryError> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DiscoveryError> {
        let beacon: Self = serde_json::from_slice(bytes)?;
        beacon.announcement.verify()?;

        let vk = ed25519_dalek::VerifyingKey::from_bytes(&beacon.announcement.verifying_key_bytes)
            .map_err(|_| IdentityError::InvalidLength)?;
        let mut payload = Vec::with_capacity(80);
        payload.extend_from_slice(&beacon.announcement.peer_id.0);
        payload.extend_from_slice(&beacon.announcement.verifying_key_bytes);
        payload.extend_from_slice(&beacon.announcement.timestamp_ms.to_be_bytes());
        payload.extend_from_slice(&beacon.tcp_port.to_be_bytes());
        payload.extend_from_slice(b"ghostmesh-beacon-v1");

        let sig_arr: [u8; ed25519_dalek::SIGNATURE_LENGTH] = beacon
            .port_signature
            .as_slice()
            .try_into()
            .map_err(|_| IdentityError::InvalidLength)?;
        let sig = ed25519_dalek::Signature::from_bytes(&sig_arr);
        use ed25519_dalek::Verifier;
        vk.verify(&payload, &sig)
            .map_err(|_| IdentityError::SignatureVerificationFailed)?;

        Ok(beacon)
    }

    pub fn send_beacon(
        &self,
        socket: &UdpSocket,
        target_addr: &str,
    ) -> Result<usize, DiscoveryError> {
        let bytes = self.encode()?;
        let sent = socket.send_to(&bytes, target_addr)?;
        Ok(sent)
    }

    pub fn recv_beacon(
        socket: &UdpSocket,
        buf: &mut [u8],
    ) -> Result<(Self, std::net::SocketAddr), DiscoveryError> {
        let (len, src_addr) = socket.recv_from(buf)?;
        let beacon = Self::decode(&buf[..len])?;
        Ok((beacon, src_addr))
    }
}

pub const DEFAULT_DISCOVERY_ADDR: &str = "0.0.0.0:45454";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub addr: std::net::SocketAddr,
    pub tcp_port: u16,
    pub timestamp_ms: u64,
}

pub fn discover_peers(
    bind_addr: &str,
    timeout: std::time::Duration,
) -> Result<Vec<DiscoveredPeer>, DiscoveryError> {
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;
    let deadline = std::time::Instant::now() + timeout;
    let mut peers = std::collections::BTreeMap::new();
    let mut buf = [0u8; 4096];

    while std::time::Instant::now() < deadline {
        match DiscoveryBeacon::recv_beacon(&socket, &mut buf) {
            Ok((beacon, addr)) => {
                let peer_id = beacon.announcement.peer_id.to_hex();
                peers.insert(
                    peer_id.clone(),
                    DiscoveredPeer {
                        peer_id,
                        addr,
                        tcp_port: beacon.tcp_port,
                        timestamp_ms: beacon.announcement.timestamp_ms,
                    },
                );
            }
            Err(DiscoveryError::Io(ref e))
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(_) => continue,
        }
    }

    Ok(peers.into_values().collect())
}

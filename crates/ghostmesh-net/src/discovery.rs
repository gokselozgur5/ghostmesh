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
}

impl DiscoveryBeacon {
    pub fn new(identity: &PeerIdentity, tcp_port: u16, timestamp_ms: u64) -> Self {
        let announcement = PeerAnnouncement::new(identity, timestamp_ms);
        Self {
            announcement,
            tcp_port,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, DiscoveryError> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DiscoveryError> {
        let beacon: Self = serde_json::from_slice(bytes)?;
        beacon.announcement.verify()?;
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

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

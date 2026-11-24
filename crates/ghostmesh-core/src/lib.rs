//! Core primitives for ghostmesh: cryptographic identity, E2EE framing, CRDTs, and transfer.

pub mod announcement;
pub mod crdt;
pub mod crypto;
pub mod identity;
pub mod transfer;

pub use announcement::PeerAnnouncement;
pub use crdt::{HlcTimestamp, LwwEntry, LwwMap, OrSet};
pub use crypto::{CryptoError, KeyExchange, SessionCipher, SessionKey};
pub use identity::{IdentityError, PeerId, PeerIdentity};
pub use transfer::{ChunkInfo, ChunkManifest, TransferError, TransferState, DEFAULT_CHUNK_SIZE};

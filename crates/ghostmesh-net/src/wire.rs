use std::io;
use thiserror::Error;

pub const MAX_FRAME_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("frame size {0} exceeds maximum allowed size {1}")]
    FrameTooLarge(usize, usize),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("unknown message kind tag: {0}")]
    UnknownKind(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageKind {
    HandshakeHello = 1,
    CrdtDelta = 2,
    FileManifest = 3,
    FileChunk = 4,
    Ping = 5,
    Pong = 6,
}

impl MessageKind {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(val: u8) -> Result<Self, WireError> {
        match val {
            1 => Ok(Self::HandshakeHello),
            2 => Ok(Self::CrdtDelta),
            3 => Ok(Self::FileManifest),
            4 => Ok(Self::FileChunk),
            5 => Ok(Self::Ping),
            6 => Ok(Self::Pong),
            other => Err(WireError::UnknownKind(other)),
        }
    }
}

use std::io::{Read, Write};

pub fn write_frame<W: Write>(writer: &mut W, data: &[u8]) -> Result<(), WireError> {
    if data.len() > MAX_FRAME_BYTES {
        return Err(WireError::FrameTooLarge(data.len(), MAX_FRAME_BYTES));
    }
    let len_prefix = (data.len() as u32).to_be_bytes();
    writer.write_all(&len_prefix)?;
    writer.write_all(data)?;
    writer.flush()?;
    Ok(())
}

pub fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, WireError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let length = u32::from_be_bytes(len_buf) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(WireError::FrameTooLarge(length, MAX_FRAME_BYTES));
    }
    let mut data = vec![0u8; length];
    reader.read_exact(&mut data)?;
    Ok(data)
}

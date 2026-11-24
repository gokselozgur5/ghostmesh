use ghostmesh_net::wire::*;
use std::io::Cursor;

#[test]
fn test_message_kind_roundtrip() {
    let kinds = [
        MessageKind::HandshakeHello,
        MessageKind::CrdtDelta,
        MessageKind::FileManifest,
        MessageKind::FileChunk,
        MessageKind::Ping,
        MessageKind::Pong,
    ];
    for kind in kinds {
        let tag = kind.to_u8();
        assert_eq!(MessageKind::from_u8(tag).unwrap(), kind);
    }
    assert!(MessageKind::from_u8(99).is_err());
}

#[test]
fn test_write_and_read_frame() {
    let payload = b"ghostmesh wire protocol verification payload";
    let mut buf = Vec::new();
    write_frame(&mut buf, payload).unwrap();
    let mut cursor = Cursor::new(buf);
    let decoded = read_frame(&mut cursor).unwrap();
    assert_eq!(decoded, payload);
}

#[test]
fn test_oversized_frame_rejection() {
    let mut invalid_header = Vec::new();
    let excessive_len = (MAX_FRAME_BYTES as u32 + 1).to_be_bytes();
    invalid_header.extend_from_slice(&excessive_len);
    let mut cursor = Cursor::new(invalid_header);
    let result = read_frame(&mut cursor);
    assert!(result.is_err());
}

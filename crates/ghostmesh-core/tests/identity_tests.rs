use ghostmesh_core::announcement::PeerAnnouncement;
use ghostmesh_core::identity::*;

#[test]
fn deterministic_seed_peer_id() {
    let seed = [7u8; 32];
    let p1 = PeerIdentity::from_seed(&seed);
    let p2 = PeerIdentity::from_seed(&seed);
    assert_eq!(p1.peer_id(), p2.peer_id());
    assert_eq!(p1.verifying_key(), p2.verifying_key());
}

#[test]
fn hex_roundtrip() {
    let seed = [42u8; 32];
    let identity = PeerIdentity::from_seed(&seed);
    let hex = identity.peer_id().to_hex();
    let decoded = PeerId::from_hex(&hex).expect("valid hex");
    assert_eq!(identity.peer_id(), decoded);
}

#[test]
fn announcement_sign_and_verify() {
    let seed = [11u8; 32];
    let identity = PeerIdentity::from_seed(&seed);
    let announcement = PeerAnnouncement::new(&identity, 1700000000000);
    assert!(announcement.verify().is_ok());

    let mut tampered = announcement.clone();
    tampered.timestamp_ms += 1;
    assert_eq!(
        tampered.verify().unwrap_err(),
        IdentityError::SignatureVerificationFailed
    );
}

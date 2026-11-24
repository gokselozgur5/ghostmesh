use ghostmesh_core::identity::PeerIdentity;
use ghostmesh_net::handshake::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

#[test]
fn test_loopback_encrypted_handshake() {
    let mut rng = rand_core::OsRng;
    let alice_identity = PeerIdentity::generate(&mut rng);
    let bob_identity = PeerIdentity::generate(&mut rng);
    let alice_id = alice_identity.peer_id();
    let bob_id = bob_identity.peer_id();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let server_addr = listener.local_addr().unwrap();

    let server_handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut rng = rand_core::OsRng;
        let mut bob_enc = server_handshake(stream, &bob_identity, &mut rng).unwrap();
        assert_eq!(bob_enc.remote_peer(), alice_id);

        let msg = bob_enc.recv_encrypted().unwrap();
        assert_eq!(msg, b"hello bob from alice");
        bob_enc.send_encrypted(b"hello alice from bob").unwrap();
    });

    let client_stream = TcpStream::connect(server_addr).unwrap();
    let mut alice_enc = client_handshake(client_stream, &alice_identity, &mut rng).unwrap();
    assert_eq!(alice_enc.remote_peer(), bob_id);

    alice_enc.send_encrypted(b"hello bob from alice").unwrap();
    let reply = alice_enc.recv_encrypted().unwrap();
    assert_eq!(reply, b"hello alice from bob");

    server_handle.join().unwrap();
}

#[test]
fn test_tampered_signature_rejection() {
    let mut rng = rand_core::OsRng;
    let alice_identity = PeerIdentity::generate(&mut rng);
    let kx = ghostmesh_core::KeyExchange::new(&mut rng);
    let mut hello = HandshakeHello::new(&alice_identity, kx.public_key(), INITIATOR_ROLE);

    // Tamper with signature
    if let Some(byte) = hello.signature.first_mut() {
        *byte ^= 0xff;
    }
    assert!(hello.verify(INITIATOR_ROLE).is_err());
}

#[test]
fn test_uks_mitm_identity_substitution_rejected() {
    let mut rng = rand_core::OsRng;
    let alice = PeerIdentity::generate(&mut rng);
    let bob = PeerIdentity::generate(&mut rng);
    let mallory = PeerIdentity::generate(&mut rng);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let bob_addr = listener.local_addr().unwrap();

    let server_handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut rng = rand_core::OsRng;
        server_handshake(stream, &bob, &mut rng)
    });

    let stream = TcpStream::connect(bob_addr).unwrap();
    let kx_alice = ghostmesh_core::KeyExchange::new(&mut rng);
    // Mallory re-signs Alice's ephemeral key with Mallory's identity
    let mallory_hello = HandshakeHello::new(&mallory, kx_alice.public_key(), INITIATOR_ROLE);
    let mut mallory_stream = stream;
    let frame = serde_json::to_vec(&mallory_hello).unwrap();
    ghostmesh_net::wire::write_frame(&mut mallory_stream, &frame).unwrap();

    // Alice expects Bob to talk to Alice, but Bob thinks he's talking to Mallory
    let bob_resp_frame = ghostmesh_net::wire::read_frame(&mut mallory_stream).unwrap();
    let bob_hello: HandshakeHello = serde_json::from_slice(&bob_resp_frame).unwrap();

    let mut alice_transcript = Vec::with_capacity(160);
    alice_transcript.extend_from_slice(&alice.peer_id().0);
    alice_transcript.extend_from_slice(alice.verifying_key().as_bytes());
    alice_transcript.extend_from_slice(kx_alice.public_key().as_bytes());
    alice_transcript.extend_from_slice(&bob_hello.peer_id.0);
    alice_transcript.extend_from_slice(&bob_hello.verifying_key_bytes);
    alice_transcript.extend_from_slice(&bob_hello.ephemeral_public);

    // Alice verifies Bob's signature with Alice's identity in transcript -> must fail because Bob signed with Mallory's identity!
    assert!(bob_hello.verify_server(&alice_transcript).is_err());

    // Close client connection so server does not hang waiting for finish frame
    drop(mallory_stream);
    let server_res = server_handle.join().unwrap();
    assert!(server_res.is_err());
}

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

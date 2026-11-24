use ghostmesh_core::identity::PeerIdentity;
use ghostmesh_net::discovery::*;
use std::net::UdpSocket;

#[test]
fn test_discovery_beacon_encode_decode_roundtrip() {
    let mut rng = rand_core::OsRng;
    let identity = PeerIdentity::generate(&mut rng);
    let beacon = DiscoveryBeacon::new(&identity, 9876, 1700000000);

    let encoded = beacon.encode().unwrap();
    let decoded = DiscoveryBeacon::decode(&encoded).unwrap();

    assert_eq!(decoded.announcement.peer_id, identity.peer_id());
    assert_eq!(decoded.tcp_port, 9876);
}

#[test]
fn test_loopback_udp_beacon_exchange() {
    let mut rng = rand_core::OsRng;
    let sender_id = PeerIdentity::generate(&mut rng);
    let beacon = DiscoveryBeacon::new(&sender_id, 8080, 1700000000);

    let receiver_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver_addr = receiver_socket.local_addr().unwrap();

    let sender_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    beacon
        .send_beacon(&sender_socket, &receiver_addr.to_string())
        .unwrap();

    let mut buf = [0u8; 1024];
    let (received_beacon, _) = DiscoveryBeacon::recv_beacon(&receiver_socket, &mut buf).unwrap();

    assert_eq!(received_beacon.announcement.peer_id, sender_id.peer_id());
    assert_eq!(received_beacon.tcp_port, 8080);
}

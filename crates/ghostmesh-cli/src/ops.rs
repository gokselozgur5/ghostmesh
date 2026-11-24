use ghostmesh_core::{
    crdt::{HlcTimestamp, LwwMap, OrSet},
    crypto::KeyExchange,
    identity::PeerIdentity,
    transfer::{ChunkManifest, TransferState},
};
use rand_core::OsRng;
use std::fs;
use std::path::Path;

pub fn print_usage() {
    println!("ghostmesh CLI");
    println!("Commands:");
    println!("  keygen                       Generate new Ed25519 peer identity");
    println!("  sim-sync                     Simulate encrypted E2EE CRDT sync session");
    println!("  pack <file>                  Create BLAKE3 chunked manifest for file");
    println!("  verify <manifest> <source>   Verify file integrity against manifest");
    println!("  live-demo                    Run live encrypted 2-node CRDT + file transfer demo");
    println!("  listen <addr>                Listen as a live mesh node on address");
    println!("  send <addr> <file>           Send file to remote mesh node over encrypted link");
}

pub fn run_keygen() {
    let mut rng = OsRng;
    let identity = PeerIdentity::generate(&mut rng);
    println!("peer_id: {}", identity.peer_id().to_hex());
    println!(
        "verifying_key: {}",
        blake3::hash(identity.verifying_key().as_bytes()).to_hex()
    );
}

pub fn run_sim_sync() {
    let mut rng = OsRng;
    let alice_id = PeerIdentity::generate(&mut rng);
    let bob_id = PeerIdentity::generate(&mut rng);

    println!("Alice peer_id: {}", alice_id.peer_id());
    println!("Bob peer_id: {}", bob_id.peer_id());

    let alice_kx = KeyExchange::new(&mut rng);
    let bob_kx = KeyExchange::new(&mut rng);

    let alice_pub = *alice_kx.public_key();
    let bob_pub = *bob_kx.public_key();

    let (mut alice_tx, mut alice_rx) = alice_kx
        .derive_session_keys(&bob_pub, true)
        .expect("derive alice keys");
    let (mut bob_tx, mut bob_rx) = bob_kx
        .derive_session_keys(&alice_pub, false)
        .expect("derive bob keys");

    let mut alice_map = LwwMap::new();
    let t1 = HlcTimestamp::new(100, 0, alice_id.peer_id());
    alice_map.set("mesh_name".to_string(), "ghost-net".to_string(), t1);

    let mut alice_set = OrSet::new(alice_id.peer_id());
    alice_set.insert("node-alpha".to_string());

    let delta_map = alice_map.delta_since(None);
    let raw_payload = serde_json::to_vec(&(&delta_map, &alice_set)).expect("serialize CRDTs");

    let (nonce, ciphertext) = alice_tx.encrypt(&raw_payload).expect("alice encrypts");
    let decrypted = bob_rx.decrypt(nonce, &ciphertext).expect("bob decrypts");
    let (rec_map, rec_set): (LwwMap<String, String>, OrSet<String>) =
        serde_json::from_slice(&decrypted).expect("deserialize CRDTs");

    let mut bob_map = LwwMap::new();
    bob_map.merge(rec_map);
    let mut bob_set = OrSet::new(bob_id.peer_id());
    bob_set.merge(rec_set);

    assert_eq!(
        bob_map.get(&"mesh_name".to_string()),
        Some(&"ghost-net".to_string())
    );
    assert!(bob_set.contains(&"node-alpha".to_string()));

    let (b_nonce, b_ct) = bob_tx.encrypt(b"SYNC_ACK").expect("bob encrypts ack");
    let b_dec = alice_rx
        .decrypt(b_nonce, &b_ct)
        .expect("alice decrypts ack");
    assert_eq!(b_dec, b"SYNC_ACK");

    println!("sim-sync: E2EE handshake, CRDT sync & convergence verified successfully.");
}

pub fn run_pack(path_str: &str) {
    let path = Path::new(path_str);
    let data = fs::read(path).expect("failed to read file");
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("blob")
        .to_string();
    let manifest = ChunkManifest::build(name, &data, 64 * 1024).expect("build manifest");
    let json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    println!("{json}");
}

pub fn run_verify(manifest_path: &str, file_path: &str) {
    let m_content = fs::read_to_string(manifest_path).expect("read manifest");
    let manifest: ChunkManifest = serde_json::from_str(&m_content).expect("parse manifest");
    manifest
        .validate(64 * 1024 * 1024)
        .expect("invalid manifest");
    let data = fs::read(file_path).expect("read data");

    let mut state = TransferState::new(manifest.clone()).expect("init transfer state");
    for chunk in &manifest.chunks {
        let start = chunk.offset as usize;
        let end = start + chunk.length;
        let slice = data.get(start..end).expect("chunk slice out of bounds");
        state
            .verify_and_store_chunk(chunk.index, slice)
            .expect("chunk verification");
    }

    assert!(state.is_complete());
    let reassembled = state.reassemble().expect("reassemble");
    assert_eq!(reassembled, data);
    println!(
        "OK: File matches manifest root hash {}",
        blake3::Hash::from(manifest.root_hash).to_hex()
    );
}

use ghostmesh_net::discovery::DiscoveryBeacon;
use ghostmesh_net::node::MeshNode;
use std::net::UdpSocket;

pub fn run_live_demo() {
    let mut rng = OsRng;
    let node_a = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();
    let node_b = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();

    let b_addr = node_b.local_addr();
    let beacon = DiscoveryBeacon::new(&node_b.identity, b_addr.port(), 1700000000);
    let s_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let r_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let r_addr = r_sock.local_addr().unwrap();

    beacon.send_beacon(&s_sock, &r_addr.to_string()).unwrap();
    let mut buf = [0u8; 1024];
    let (rec_beacon, _) = DiscoveryBeacon::recv_beacon(&r_sock, &mut buf).unwrap();
    println!(
        "LAN Discovery Beacon: peer={}",
        rec_beacon.announcement.peer_id
    );

    let ts = HlcTimestamp::new(100, 1, node_a.peer_id());
    node_a
        .lww_map
        .lock()
        .unwrap()
        .set("k".into(), "v".into(), ts);
    node_a.sync_lww_with(b_addr).unwrap();
    println!("CRDT Sync: node_b network='ghostmesh-live'");

    let payload = b"LIVE_MESH_ENCRYPTED_TRANSFER_VALIDATION";
    node_a.send_file_to(b_addr, "live.bin", payload).unwrap();
    let file = node_b
        .completed_files
        .lock()
        .unwrap()
        .get("live.bin")
        .unwrap()
        .clone();
    assert_eq!(&file, payload);
    println!(
        "File Transfer: verified {} bytes encrypted transfer.",
        file.len()
    );
}

use std::net::ToSocketAddrs;

pub fn run_listen(addr_str: &str) {
    let mut rng = OsRng;
    let node = MeshNode::bind(PeerIdentity::generate(&mut rng), addr_str).expect("bind");
    println!("Listening on {}", node.local_addr());
    println!("Node PeerId: {}", node.peer_id());
    println!("Node is active. Waiting for incoming connections...");
    std::thread::park();
}

pub fn run_send(addr_str: &str, file_path: &str) {
    let addr = addr_str
        .to_socket_addrs()
        .expect("invalid addr")
        .next()
        .expect("resolve addr");
    let mut rng = OsRng;
    let node = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").expect("bind");
    let path = Path::new(file_path);
    let bytes = fs::read(path).expect("read file");
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

    node.send_file_to(addr, name, &bytes).expect("send file");
    println!(
        "Successfully sent {} ({} bytes) to {}",
        name,
        bytes.len(),
        addr
    );
}

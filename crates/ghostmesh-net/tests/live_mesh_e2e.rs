use ghostmesh_core::crdt::HlcTimestamp;
use ghostmesh_core::identity::PeerIdentity;
use ghostmesh_net::node::MeshNode;

#[test]
fn test_two_node_crdt_convergence() {
    let mut rng = rand_core::OsRng;
    let node_a = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();
    let node_b = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();

    let ts1 = HlcTimestamp::new(100, 1, node_a.peer_id());
    let ts2 = HlcTimestamp::new(200, 1, node_b.peer_id());

    node_a
        .lww_map
        .lock()
        .unwrap()
        .set("title".into(), "Mesh v1".into(), ts1);
    node_b
        .lww_map
        .lock()
        .unwrap()
        .set("title".into(), "Mesh v2".into(), ts2);
    node_b
        .lww_map
        .lock()
        .unwrap()
        .set("status".into(), "active".into(), ts1);

    node_a.sync_lww_with(node_b.local_addr()).unwrap();

    let map_a = node_a.lww_map.lock().unwrap();
    let map_b = node_b.lww_map.lock().unwrap();

    assert_eq!(map_a.get(&"title".to_string()).unwrap(), "Mesh v2");
    assert_eq!(map_b.get(&"title".to_string()).unwrap(), "Mesh v2");
    assert_eq!(map_a.get(&"status".to_string()).unwrap(), "active");
    assert_eq!(map_b.get(&"status".to_string()).unwrap(), "active");
}

#[test]
fn test_three_node_multi_hop_crdt_convergence() {
    let mut rng = rand_core::OsRng;
    let node_a = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();
    let node_b = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();
    let node_c = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();

    let ts_a = HlcTimestamp::new(100, 0, node_a.peer_id());
    let ts_c = HlcTimestamp::new(100, 0, node_c.peer_id());

    node_a
        .lww_map
        .lock()
        .unwrap()
        .set("node_a_key".into(), "from_a".into(), ts_a);
    node_c
        .lww_map
        .lock()
        .unwrap()
        .set("node_c_key".into(), "from_c".into(), ts_c);

    // Sync A <-> B
    node_a.sync_lww_with(node_b.local_addr()).unwrap();
    // Sync B <-> C
    node_b.sync_lww_with(node_c.local_addr()).unwrap();
    // Sync A <-> B again so A receives C's delta through B!
    node_a.sync_lww_with(node_b.local_addr()).unwrap();

    let map_a = node_a.lww_map.lock().unwrap();
    let map_b = node_b.lww_map.lock().unwrap();
    let map_c = node_c.lww_map.lock().unwrap();

    assert_eq!(map_a.get(&"node_c_key".to_string()).unwrap(), "from_c");
    assert_eq!(map_b.get(&"node_a_key".to_string()).unwrap(), "from_a");
    assert_eq!(map_c.get(&"node_a_key".to_string()).unwrap(), "from_a");
}

#[test]
fn test_live_chunked_encrypted_file_transfer() {
    let mut rng = rand_core::OsRng;
    let sender_node = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();
    let receiver_node = MeshNode::bind(PeerIdentity::generate(&mut rng), "127.0.0.1:0").unwrap();

    let test_file_name = "test_data.bin";
    // 150 KiB file spans multiple 64 KiB chunks
    let original_bytes = vec![0x42u8; 150 * 1024];

    sender_node
        .send_file_to(receiver_node.local_addr(), test_file_name, &original_bytes)
        .unwrap();

    let completed = receiver_node.completed_files.lock().unwrap();
    let received_bytes = completed.get(test_file_name).expect("file not found in receiver");
    assert_eq!(received_bytes, &original_bytes);
}

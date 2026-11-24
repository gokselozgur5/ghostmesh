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

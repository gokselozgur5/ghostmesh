use ghostmesh_core::crdt::{HlcTimestamp, LwwMap, OrSet};
use ghostmesh_core::identity::PeerId;

#[test]
fn hlc_causality_and_ordering() {
    let p1 = PeerId([1u8; 32]);
    let p2 = PeerId([2u8; 32]);

    let t1 = HlcTimestamp::new(1000, 0, p1);
    let t2 = t1.tick(1000, p1);
    assert_eq!(t2.physical_ms, 1000);
    assert_eq!(t2.logical, 1);
    assert!(t2 > t1);

    let t3 = t2.merge(&HlcTimestamp::new(1050, 0, p2), 1020, p1);
    assert_eq!(t3.physical_ms, 1050);
    assert_eq!(t3.logical, 1);
    assert!(t3 > t2);
}

#[test]
fn lww_map_convergence_and_properties() {
    let p1 = PeerId([1u8; 32]);
    let p2 = PeerId([2u8; 32]);

    let mut map_a = LwwMap::new();
    let mut map_b = LwwMap::new();

    let t1 = HlcTimestamp::new(1000, 0, p1);
    let t2 = HlcTimestamp::new(2000, 0, p2);

    map_a.set("k1".to_string(), "v1_older".to_string(), t1);
    map_b.set("k1".to_string(), "v1_newer".to_string(), t2);

    // Merge A into B and B into A
    let mut a_copy = map_a.clone();
    a_copy.merge(map_b.clone());

    let mut b_copy = map_b.clone();
    b_copy.merge(map_a.clone());

    // Commutativity
    assert_eq!(a_copy, b_copy);
    assert_eq!(a_copy.get(&"k1".to_string()), Some(&"v1_newer".to_string()));

    // Idempotence
    let mut idemp = a_copy.clone();
    idemp.merge(a_copy.clone());
    assert_eq!(idemp, a_copy);

    // Tombstone deletion
    let t3 = HlcTimestamp::new(3000, 0, p1);
    a_copy.delete("k1".to_string(), t3);
    assert_eq!(a_copy.get(&"k1".to_string()), None);
}

#[test]
fn or_set_add_wins_concurrency() {
    let p1 = PeerId([1u8; 32]);
    let p2 = PeerId([2u8; 32]);

    let mut set_a = OrSet::new(p1);
    let mut set_b = OrSet::new(p2);

    set_a.insert("shared_item".to_string());
    set_b.merge(set_a.clone());
    assert!(set_b.contains(&"shared_item".to_string()));

    // Concurrent: B removes item, A re-adds item with new tag
    set_b.remove(&"shared_item".to_string());
    assert!(!set_b.contains(&"shared_item".to_string()));

    set_a.insert("shared_item".to_string());

    // Merge: Add wins over remove because A generated a fresh tag unseen by B's removal
    let mut merged = set_b;
    merged.merge(set_a);
    assert!(merged.contains(&"shared_item".to_string()));
}

#[test]
fn hlc_logical_overflow_carries_to_physical() {
    let p1 = PeerId([1u8; 32]);
    let p2 = PeerId([2u8; 32]);

    let t1 = HlcTimestamp::new(1000, u16::MAX, p1);
    let t2 = t1.tick(1000, p1);
    assert_eq!(t2.physical_ms, 1001);
    assert_eq!(t2.logical, 0);
    assert!(t2 > t1);

    let t_remote = HlcTimestamp::new(1000, u16::MAX, p2);
    let t_merged = t1.merge(&t_remote, 1000, p1);
    assert_eq!(t_merged.physical_ms, 1001);
    assert_eq!(t_merged.logical, 0);
    assert!(t_merged > t1);
    assert!(t_merged > t_remote);
}

#[test]
fn or_set_merge_does_not_adopt_foreign_next_seq() {
    let p1 = PeerId([1u8; 32]);
    let p2 = PeerId([2u8; 32]);

    let mut set_a = OrSet::new(p1);
    let mut set_b = OrSet::new(p2);

    // B inserts with high sequence number
    for _ in 0..10 {
        set_b.insert("b_item".to_string());
    }

    set_a.merge(set_b);
    let tag = set_a.insert("a_item".to_string());
    // set_a should have local sequence 1, not 11
    assert_eq!(tag.peer_id, p1);
    assert_eq!(tag.seq, 1);
}

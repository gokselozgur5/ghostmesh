use ghostmesh_core::crypto::{CryptoError, KeyExchange, SessionCipher, SessionKey};
use rand_core::OsRng;

#[test]
fn e2ee_handshake_and_bidirectional_encryption() {
    let mut rng = OsRng;
    let alice_kx = KeyExchange::new(&mut rng);
    let bob_kx = KeyExchange::new(&mut rng);

    let alice_pub = *alice_kx.public_key();
    let bob_pub = *bob_kx.public_key();

    let (mut alice_tx, mut alice_rx) = alice_kx.derive_session_keys(&bob_pub, true);
    let (mut bob_tx, mut bob_rx) = bob_kx.derive_session_keys(&alice_pub, false);

    // Alice -> Bob
    let (nonce, ciphertext) = alice_tx.encrypt(b"hello from alice").unwrap();
    assert_eq!(nonce, 0);
    let decrypted = bob_rx.decrypt(nonce, &ciphertext).unwrap();
    assert_eq!(decrypted, b"hello from alice");

    // Bob -> Alice
    let (b_nonce, b_ct) = bob_tx.encrypt(b"reply from bob").unwrap();
    assert_eq!(b_nonce, 0);
    let b_dec = alice_rx.decrypt(b_nonce, &b_ct).unwrap();
    assert_eq!(b_dec, b"reply from bob");
}

#[test]
fn replay_and_out_of_order_protection() {
    let key = SessionKey([99u8; 32]);
    let mut tx = SessionCipher::new(key.clone());
    let mut rx = SessionCipher::new(key);

    let (n0, c0) = tx.encrypt(b"msg 0").unwrap();
    let (n1, c1) = tx.encrypt(b"msg 1").unwrap();

    // Replay detection: delivering n1 first fails with ReplayOrOutOfOrder
    let err = rx.decrypt(n1, &c1).unwrap_err();
    assert_eq!(err, CryptoError::ReplayOrOutOfOrder(1, 0));

    // Deliver n0 successfully
    assert_eq!(rx.decrypt(n0, &c0).unwrap(), b"msg 0");

    // Replay of n0 must fail
    let replay_err = rx.decrypt(n0, &c0).unwrap_err();
    assert_eq!(replay_err, CryptoError::ReplayOrOutOfOrder(0, 1));

    // Deliver n1 successfully
    assert_eq!(rx.decrypt(n1, &c1).unwrap(), b"msg 1");
}

#[test]
fn tamper_and_wrong_key_detection() {
    let key1 = SessionKey([1u8; 32]);
    let key2 = SessionKey([2u8; 32]);
    let mut tx = SessionCipher::new(key1);
    let mut rx_wrong = SessionCipher::new(key2);

    let (nonce, mut ciphertext) = tx.encrypt(b"tamper test").unwrap();
    // Wrong key
    assert_eq!(
        rx_wrong.decrypt(nonce, &ciphertext).unwrap_err(),
        CryptoError::DecryptionFailed
    );

    // Tampered payload with correct key
    let mut rx_correct = SessionCipher::new(SessionKey([1u8; 32]));
    ciphertext[0] ^= 0xff;
    assert_eq!(
        rx_correct.decrypt(nonce, &ciphertext).unwrap_err(),
        CryptoError::DecryptionFailed
    );
}

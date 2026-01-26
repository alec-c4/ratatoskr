use rand::thread_rng;
use ratatoskr_core::models::RatchetHeader;
use ratatoskr_core::ratchet::DoubleRatchetSession;
use x25519_dalek::{PublicKey, StaticSecret};

#[test]
fn test_double_ratchet_basic_flow() {
    let mut rng = thread_rng();

    // 1. Setup X3DH (Simulated)
    // Alice and Bob share a secret "SK" via X3DH
    let shared_secret = [7u8; 32]; // Arbitrary shared secret

    // Bob has a signed prekey (or just his identity key + ephemeral)
    // For this basic ratchet test, we assume Bob has a keypair that Alice knows the pubkey of.
    let bob_static = StaticSecret::random_from_rng(&mut rng);
    let bob_public = PublicKey::from(&bob_static);

    // 2. Init Sessions
    let mut alice_session = DoubleRatchetSession::new_alice(shared_secret, bob_public);
    let mut bob_session = DoubleRatchetSession::new_bob(shared_secret, bob_static);

    // 3. Alice sends Message 1
    let msg1 = b"Hello Bob!";
    let (header1, cipher1) = alice_session.encrypt(msg1).unwrap();

    // 4. Bob receives Message 1
    let decrypted1 = bob_session.decrypt(&header1, &cipher1).unwrap();
    assert_eq!(msg1.to_vec(), decrypted1);

    // 5. Bob sends Reply 1
    let reply1 = b"Hi Alice!";
    let (header2, cipher2) = bob_session.encrypt(reply1).unwrap();

    // 6. Alice receives Reply 1
    let decrypted2 = alice_session.decrypt(&header2, &cipher2).unwrap();
    assert_eq!(reply1.to_vec(), decrypted2);

    // 7. Alice sends Message 2
    let msg2 = b"How are you?";
    let (header3, cipher3) = alice_session.encrypt(msg2).unwrap();

    // 8. Alice sends Message 3
    let msg3 = b"Are you there?";
    let (header4, cipher4) = alice_session.encrypt(msg3).unwrap();

    // 9. Bob receives Message 3 FIRST (out of order)
    let decrypted3 = bob_session.decrypt(&header4, &cipher4).unwrap();
    assert_eq!(msg3.to_vec(), decrypted3);

    // 10. Bob receives Message 2
    let decrypted2_late = bob_session.decrypt(&header3, &cipher3).unwrap();
    assert_eq!(msg2.to_vec(), decrypted2_late);
}

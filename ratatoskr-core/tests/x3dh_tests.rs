use rand::rngs::OsRng;
use ratatoskr_core::key_vault::KeyVault;
use ratatoskr_core::x3dh::{self, PreKeyBundle};
use x25519_dalek::{PublicKey, StaticSecret};

#[test]
fn test_x3dh_and_ratchet_flow() {
    // 1. Bob Setup
    // Bob has a long-term identity (KeyVault now has both Ed25519 and X25519)
    let bob_vault = KeyVault::generate_random();

    // Bob generates Signed Prekey
    let bob_spk_secret = StaticSecret::random_from_rng(OsRng);

    // Bob generates One-time Prekey
    let bob_opk_secret = StaticSecret::random_from_rng(OsRng);

    // Bob creates Bundle
    let bob_bundle = PreKeyBundle::new(
        &bob_vault.dh_identity,
        &bob_vault.signing_key,
        &bob_spk_secret,
        Some(&bob_opk_secret),
    );

    // 2. Alice Setup
    let alice_vault = KeyVault::generate_random();
    let alice_ik_pub = PublicKey::from(&alice_vault.dh_identity);

    // 3. Alice initiates
    let msg_to_bob = b"Hello Bob, this is Alice via X3DH!";
    let alice_result = x3dh::initialize_alice(
        &alice_vault.dh_identity,
        alice_ik_pub,
        &bob_bundle,
        &bob_vault.signing_key.verifying_key(), // Alice knows Bob's Ed25519 key
        msg_to_bob,
    )
    .expect("Alice failed to init X3DH");

    // 4. Bob receives
    let (mut bob_session, decrypted_msg) = x3dh::initialize_bob(
        &bob_vault.dh_identity,
        &bob_spk_secret,
        Some(&bob_opk_secret),
        alice_ik_pub,
        alice_result.ephemeral_key,
        &alice_result.initial_header,
        &alice_result.initial_ciphertext,
    )
    .expect("Bob failed to init X3DH");

    assert_eq!(msg_to_bob.to_vec(), decrypted_msg);

    // 5. Continue conversation (Double Ratchet)

    // Bob replies
    let msg_from_bob = b"Hi Alice! Secure channel established.";
    let (header_b, cipher_b) = bob_session.encrypt(msg_from_bob).unwrap();

    // Alice decrypts
    let mut alice_session = alice_result.session;
    let decrypted_reply = alice_session.decrypt(&header_b, &cipher_b).unwrap();

    assert_eq!(msg_from_bob.to_vec(), decrypted_reply);
}

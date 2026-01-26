use rand::rngs::OsRng;
use ratatoskr_core::key_vault::KeyVault;
use ratatoskr_core::messaging::MessagingService;
use ratatoskr_core::storage::Storage;
use ratatoskr_core::x3dh::PreKeyBundle;
use tempfile::tempdir;
use x25519_dalek::{PublicKey, StaticSecret};

#[tokio::test]
async fn test_messaging_service_flow() {
    let dir = tempdir().unwrap();
    let db_path_alice = dir.path().join("alice.db");
    let db_path_bob = dir.path().join("bob.db");

    // 1. Setup Alice
    let alice_vault = KeyVault::generate_random();
    let alice_storage = Storage::init(&db_path_alice).await.unwrap();
    let alice_service = MessagingService::new(&alice_storage, &alice_vault);

    // 2. Setup Bob
    let bob_vault = KeyVault::generate_random();
    let bob_storage = Storage::init(&db_path_bob).await.unwrap();
    let bob_service = MessagingService::new(&bob_storage, &bob_vault);

    // 3. Bob generates and publishes PreKeys (Simulated)
    let bob_spk_secret = StaticSecret::random_from_rng(OsRng);
    let bob_opk_secret = StaticSecret::random_from_rng(OsRng);

    // Bob must save these to his storage so he can decrypt later
    bob_storage
        .save_signed_prekey(&bob_spk_secret)
        .await
        .unwrap();
    bob_storage
        .save_onetime_prekey(&bob_opk_secret)
        .await
        .unwrap();

    let bob_bundle = PreKeyBundle::new(
        &bob_vault.dh_identity,
        &bob_vault.signing_key,
        &bob_spk_secret,
        Some(&bob_opk_secret),
    );

    let bob_did = "did:rata:bob"; // Mock DID
    let alice_did = "did:rata:alice"; // Mock DID

    // 4. Alice sends Message 1 (X3DH Init)
    let msg1_text = b"Hello Bob, this is the first message.";
    let encrypted_msg1 = alice_service
        .encrypt_message(
            bob_did,
            Some(&bob_vault.signing_key.verifying_key()),
            Some(&bob_bundle),
            msg1_text,
        )
        .await
        .expect("Alice encryption failed");

    // 5. Bob receives Message 1
    let decrypted_msg1 = bob_service
        .decrypt_message(alice_did, encrypted_msg1)
        .await
        .expect("Bob decryption failed");

    assert_eq!(msg1_text.to_vec(), decrypted_msg1);

    // 6. Bob replies (Session Established)
    let msg2_text = b"Hello Alice! I read you loud and clear.";
    // Bob doesn't need a bundle for Alice because the session is established.
    let encrypted_msg2 = bob_service
        .encrypt_message(
            alice_did, None, // No new session needed
            None, msg2_text,
        )
        .await
        .expect("Bob encryption failed");

    // 7. Alice receives Reply
    let decrypted_msg2 = alice_service
        .decrypt_message(bob_did, encrypted_msg2)
        .await
        .expect("Alice decryption failed");

    assert_eq!(msg2_text.to_vec(), decrypted_msg2);
}

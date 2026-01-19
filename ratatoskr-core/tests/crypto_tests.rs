use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use rand::rngs::OsRng;
use ratatoskr_core::crypto::encrypt_sos_signal;
use ratatoskr_core::models::{GeoLocation, SosPayload, SosType};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

#[test]
fn test_sos_encryption_decryption_flow() {
    // 1. Setup: Create "Organization" keys (Receiver)
    let org_secret = StaticSecret::random_from_rng(OsRng);
    let org_public = PublicKey::from(&org_secret);
    let org_pub_bytes = *org_public.as_bytes();

    // 2. Create Payload
    let payload = SosPayload {
        help_type: SosType::Medical,
        location: Some(GeoLocation {
            latitude: 55.0,
            longitude: 37.0,
            accuracy: 10.0,
        }),
        description: "Test SOS".to_string(),
        timestamp: 123456789,
    };

    // 3. Encrypt (Client Side)
    let packet = encrypt_sos_signal(&payload, &org_pub_bytes).expect("Encryption failed");

    // 4. Decrypt (Organization Side)
    // Reconstruct Sender Public Key
    let sender_public_bytes: [u8; 32] = packet.ephemeral_public_key.clone().try_into().unwrap();
    let sender_public = PublicKey::from(sender_public_bytes);

    // Derive Shared Secret
    let shared_secret = org_secret.diffie_hellman(&sender_public);

    // KDF (Must match implementation in crypto.rs)
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.as_bytes());
    let aes_key_bytes = hasher.finalize();
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&aes_key_bytes);

    // Decrypt AES
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&packet.nonce);
    let plaintext = cipher
        .decrypt(nonce, packet.ciphertext.as_ref())
        .expect("Decryption failed");

    // 5. Verify Content
    let decrypted_payload: SosPayload = serde_json::from_slice(&plaintext).unwrap();
    assert_eq!(decrypted_payload.description, "Test SOS");
    assert!(matches!(decrypted_payload.help_type, SosType::Medical));
}

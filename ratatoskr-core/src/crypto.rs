use crate::models::{EncryptedSosPacket, SosPayload};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm,
    Nonce, // Or `Key`
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{EphemeralSecret, PublicKey};

// Keypair generation (placeholder from previous step, can be kept or expanded)
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    (vec![], vec![])
}

/// Encrypts an SOS signal using the ECIES scheme (Anonymous encryption)
/// Sender Ephemeral Priv + Receiver Static Pub -> Shared Secret -> AES Key
pub fn encrypt_sos_signal(
    payload: &SosPayload,
    trusted_public_key_bytes: &[u8; 32],
) -> Result<EncryptedSosPacket, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Generate a one-time (ephemeral) key for this specific transmission
    let sender_secret = EphemeralSecret::random_from_rng(OsRng);
    let sender_public = PublicKey::from(&sender_secret);

    // 2. Restore the recipient's public key (Organization)
    let receiver_public = PublicKey::from(*trusted_public_key_bytes);

    // 3. Calculate the shared secret (Diffie-Hellman)
    let shared_secret = sender_secret.diffie_hellman(&receiver_public);

    // 4. Transform the Shared Secret into a symmetric key for AES (via SHA-256)
    // This is standard practice (KDF) to distribute entropy evenly
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.as_bytes());
    let aes_key_bytes = hasher.finalize();
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&aes_key_bytes);

    // 5. Encrypt data
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12]; // 96-bit nonce
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = serde_json::to_vec(payload)?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failure: {}", e))?;

    // 6. Form the packet
    Ok(EncryptedSosPacket {
        ephemeral_public_key: sender_public.as_bytes().to_vec(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

use crate::models::RatchetHeader;
use crate::ratchet::DoubleRatchetSession;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

const X3DH_INFO: &[u8] = b"RatatoskrV1 X3DH";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PreKeyBundle {
    pub identity_key: PublicKey,           // IK_B
    pub signed_prekey: PublicKey,          // SPK_B
    pub onetime_prekey: Option<PublicKey>, // OPK_B
    pub signature: Vec<u8>,                // Signature over (IK_B || SPK_B) by Ed_B
}

impl PreKeyBundle {
    /// Bob generates a bundle
    pub fn new(
        identity_secret: &StaticSecret,
        identity_signing_key: &SigningKey,
        signed_prekey_secret: &StaticSecret,
        onetime_prekey_secret: Option<&StaticSecret>,
    ) -> Self {
        let ik = PublicKey::from(identity_secret);
        let spk = PublicKey::from(signed_prekey_secret);

        // Sign (IK || SPK)
        let mut msg = Vec::new();
        msg.extend_from_slice(ik.as_bytes());
        msg.extend_from_slice(spk.as_bytes());

        let signature = identity_signing_key.sign(&msg);

        let opk = onetime_prekey_secret.map(PublicKey::from);

        Self {
            identity_key: ik,
            signed_prekey: spk,
            onetime_prekey: opk,
            signature: signature.to_vec(),
        }
    }

    /// Verify the bundle signature using Bob's trusted Ed25519 ID
    pub fn verify(&self, bob_verifying_key: &VerifyingKey) -> bool {
        let mut msg = Vec::new();
        msg.extend_from_slice(self.identity_key.as_bytes());
        msg.extend_from_slice(self.signed_prekey.as_bytes());

        if let Ok(sig) = Signature::from_slice(&self.signature) {
            bob_verifying_key.verify(&msg, &sig).is_ok()
        } else {
            false
        }
    }
}

/// Result of the X3DH exchange on Alice's side
pub struct X3dhInitResult {
    pub session: DoubleRatchetSession,
    pub initial_header: RatchetHeader,
    pub initial_ciphertext: Vec<u8>,
    pub ephemeral_key: PublicKey, // EK_A (sent in the clear usually alongside ciphertext)
    pub used_opk: Option<PublicKey>, // To tell Bob which OPK was used (if any)
}

/// Alice initializes the session
pub fn initialize_alice(
    alice_identity_secret: &StaticSecret,
    _alice_identity_public: PublicKey, // IK_A
    bob_bundle: &PreKeyBundle,
    bob_ed25519_public_key: &VerifyingKey,
    initial_message_plaintext: &[u8],
) -> Result<X3dhInitResult, String> {
    // 1. Verify Bob's bundle
    if !bob_bundle.verify(bob_ed25519_public_key) {
        return Err("Invalid PreKeyBundle signature".to_string());
    }

    // 2. Generate Ephemeral Key EK_A
    let ek_a_secret = StaticSecret::random_from_rng(OsRng);
    let ek_a_public = PublicKey::from(&ek_a_secret);

    // 3. Compute DHs
    // DH1 = DH(IK_A, SPK_B)
    let dh1 = alice_identity_secret.diffie_hellman(&bob_bundle.signed_prekey);

    // DH2 = DH(EK_A, IK_B)
    let dh2 = ek_a_secret.diffie_hellman(&bob_bundle.identity_key);

    // DH3 = DH(EK_A, SPK_B)
    let dh3 = ek_a_secret.diffie_hellman(&bob_bundle.signed_prekey);

    // DH4 = DH(EK_A, OPK_B) if present
    let mut dh4_opt = None;
    if let Some(opk) = bob_bundle.onetime_prekey {
        dh4_opt = Some(ek_a_secret.diffie_hellman(&opk));
    }

    // 4. KDF
    let mut input = Vec::new();
    input.extend_from_slice(dh1.as_bytes());
    input.extend_from_slice(dh2.as_bytes());
    input.extend_from_slice(dh3.as_bytes());
    if let Some(dh4) = dh4_opt {
        input.extend_from_slice(dh4.as_bytes());
    }

    let mut hasher = Sha256::new();
    hasher.update(X3DH_INFO);
    hasher.update(&input);
    let sk: [u8; 32] = hasher.finalize().into();

    // 5. Init Double Ratchet
    // Alice computes SK.
    // In DR, Alice needs `bob_dh_public_key`. Which one?
    // Usually the Signed PreKey is used as the initial Ratchet Key, or Bob's part of the first ratchet.
    // Spec says: "Alice then uses SK as the initial root key ... and starts a Double Ratchet session."
    // DoubleRatchetSession::new_alice(sk, bob_dh_public_key)
    // The `bob_dh_public_key` passed to new_alice is Bob's current Ratchet Public Key.
    // Signal uses the Signed PreKey as Bob's initial ratchet key.

    let mut session = DoubleRatchetSession::new_alice(sk, bob_bundle.signed_prekey);

    // 6. Encrypt initial message
    let (header, ciphertext) = session.encrypt(initial_message_plaintext)?;

    Ok(X3dhInitResult {
        session,
        initial_header: header,
        initial_ciphertext: ciphertext,
        ephemeral_key: ek_a_public,
        used_opk: bob_bundle.onetime_prekey,
    })
}

/// Bob processes the initial message
pub fn initialize_bob(
    bob_identity_secret: &StaticSecret,
    bob_signed_prekey_secret: &StaticSecret,
    bob_onetime_prekey_secret: Option<&StaticSecret>,
    alice_identity_public: PublicKey, // IK_A (Alice includes this in the header usually, or Bob knows it)
    alice_ephemeral_key: PublicKey,   // EK_A
    alice_initial_header: &RatchetHeader,
    alice_initial_ciphertext: &[u8],
) -> Result<(DoubleRatchetSession, Vec<u8>), String> {
    // 1. Compute DHs
    // DH1 = DH(IK_A, SPK_B)  <-- Bob uses SPK secret + IK_A
    let dh1 = bob_signed_prekey_secret.diffie_hellman(&alice_identity_public);

    // DH2 = DH(EK_A, IK_B)   <-- Bob uses IK secret + EK_A
    let dh2 = bob_identity_secret.diffie_hellman(&alice_ephemeral_key);

    // DH3 = DH(EK_A, SPK_B)  <-- Bob uses SPK secret + EK_A
    let dh3 = bob_signed_prekey_secret.diffie_hellman(&alice_ephemeral_key);

    // DH4 = DH(EK_A, OPK_B)
    let mut dh4_opt = None;
    if let Some(opk_secret) = bob_onetime_prekey_secret {
        dh4_opt = Some(opk_secret.diffie_hellman(&alice_ephemeral_key));
    }

    // 2. KDF
    let mut input = Vec::new();
    input.extend_from_slice(dh1.as_bytes());
    input.extend_from_slice(dh2.as_bytes());
    input.extend_from_slice(dh3.as_bytes());
    if let Some(dh4) = dh4_opt {
        input.extend_from_slice(dh4.as_bytes());
    }

    let mut hasher = Sha256::new();
    hasher.update(X3DH_INFO);
    hasher.update(&input);
    let sk: [u8; 32] = hasher.finalize().into();

    // 3. Init Session
    // Bob needs to supply the keypair for the initial Ratchet Key, which is his Signed Prekey.
    // Clone secret to pass ownership or use ref? StaticSecret is usually not copy.
    // DoubleRatchetSession::new_bob(shared_secret, bob_dh_key_pair)
    // We need to clone the secret bytes to create a new StaticSecret if needed,
    // or just pass it if new_bob takes it.
    // Our `new_bob` takes `bob_dh_key_pair: StaticSecret`.

    let bob_dh_key_pair = StaticSecret::from(bob_signed_prekey_secret.to_bytes());
    let mut session = DoubleRatchetSession::new_bob(sk, bob_dh_key_pair);

    // 4. Decrypt
    let plaintext = session.decrypt(alice_initial_header, alice_initial_ciphertext)?;

    Ok((session, plaintext))
}

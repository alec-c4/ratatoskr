use crate::models::RatchetHeader;
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sha2::Sha256;
use std::collections::HashMap;
use x25519_dalek::{PublicKey, StaticSecret};

type SkippedMessageKeys = HashMap<(Vec<u8>, u32), [u8; 32]>;
type SkippedMessageKeysEntry = ((Vec<u8>, u32), [u8; 32]);

// Constants
const SKIPPED_MESSAGE_KEYS_MAX: usize = 500;
const HKDF_INFO_ROOT: &[u8] = b"RatatoskrV1 Root";

#[derive(Serialize, Deserialize)]
pub struct DoubleRatchetSession {
    // State
    #[serde(with = "serde_static_secret")]
    pub dhs: StaticSecret, // Our current DH key pair
    pub dhr: Option<PublicKey>, // Their current DH public key (None if we haven't received it yet)

    pub rk: [u8; 32],           // Root Key
    pub ck_s: [u8; 32],         // Chain Key (Sending)
    pub ck_r: Option<[u8; 32]>, // Chain Key (Receiving)

    pub n_s: u32, // Message number (Sending)
    pub n_r: u32, // Message number (Receiving)
    pub pn: u32,  // Previous chain length

    // Skipped message keys for out-of-order messages
    // (DH_pub, N) -> Message Key
    // PublicKey needs to be hashable for HashMap key. x25519 PublicKey is not Hash by default?
    // We'll use bytes for the key.
    #[serde(with = "serde_skipped_keys")]
    pub skipped_message_keys: SkippedMessageKeys,
}

mod serde_static_secret {
    use super::*;

    pub fn serialize<S>(secret: &StaticSecret, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // StaticSecret::to_bytes() returns [u8; 32]
        let bytes = secret.to_bytes();
        // Serialize as a byte array (or sequence)
        // Since [u8; 32] is standard, we can verify how serde handles it.
        // But to be safe and compatible with standard bytes serialization:
        serializer.serialize_bytes(&bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StaticSecret, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We expect bytes.
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        if bytes.len() != 32 {
            return Err(de::Error::custom("StaticSecret must be 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(StaticSecret::from(arr))
    }
}

mod serde_skipped_keys {
    use super::*;

    // We need to serialize the HashMap. Keys are (Vec<u8>, u32).
    // Default serde impl for HashMap works if Key implements Serialize/Deserialize.
    // Vec<u8> and u32 do.
    // So we might not need this module if we just use HashMap<(Vec<u8>, u32), [u8; 32]>
    // But let's keep the type as is.

    pub fn serialize<S>(map: &SkippedMessageKeys, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to a vector of tuples to ensure consistent serialization if needed,
        // or just let serde handle the map.
        // Map keys in JSON must be strings. If this goes to JSON, (Vec<u8>, u32) as key will fail.
        // If it goes to bincode/cbor, it's fine.
        // Assuming this might be JSON for debug or generic storage:
        // We'll serialize as a list of entries.
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (k, v) in map {
            seq.serialize_element(&(k, v))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SkippedMessageKeys, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<SkippedMessageKeysEntry> = Deserialize::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}

impl DoubleRatchetSession {
    pub fn new_alice(shared_secret: [u8; 32], bob_dh_public_key: PublicKey) -> Self {
        // Alice starts with a fresh DH key
        let dhs = StaticSecret::random_from_rng(rand::thread_rng());
        let dh_out = kdf_dh(&dhs, &bob_dh_public_key);

        let (rk, ck_s) = kdf_rk(&shared_secret, &dh_out);

        Self {
            dhs,
            dhr: Some(bob_dh_public_key),
            rk,
            ck_s,
            ck_r: None, // Will be initialized when Bob replies
            n_s: 0,
            n_r: 0,
            pn: 0,
            skipped_message_keys: HashMap::new(),
        }
    }

    pub fn new_bob(
        shared_secret: [u8; 32],
        bob_dh_key_pair: StaticSecret, // The one Alice used to init
    ) -> Self {
        Self {
            dhs: bob_dh_key_pair,
            dhr: None, // Will get Alice's key in the first message
            rk: shared_secret,
            ck_s: [0u8; 32], // Placeholder, will be rotated immediately
            ck_r: None,
            n_s: 0,
            n_r: 0,
            pn: 0,
            skipped_message_keys: HashMap::new(),
        }
    }

    // Encrypt a message
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<(RatchetHeader, Vec<u8>), String> {
        let (ck_new, mk) = kdf_ck(&self.ck_s);
        self.ck_s = ck_new;

        let header = RatchetHeader {
            dh_pub: PublicKey::from(&self.dhs),
            n: self.n_s,
            pn: self.pn,
        };

        self.n_s += 1;

        let ciphertext = encrypt_message(&mk, plaintext, &header)?;

        Ok((header, ciphertext))
    }

    // Decrypt a message
    pub fn decrypt(
        &mut self,
        header: &RatchetHeader,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, String> {
        // 1. Try skipped message keys
        let header_pub_bytes = header.dh_pub.as_bytes().to_vec();
        if let Some(mk) = self
            .skipped_message_keys
            .remove(&(header_pub_bytes.clone(), header.n))
        {
            return decrypt_message(&mk, ciphertext, header);
        }

        // 2. Check if DHR changed (Ratchet Step)
        if let Some(dhr) = self.dhr {
            if header.dh_pub != dhr {
                // Determine if this is a new chain
                // We need to advance the chain to catch up with previous messages if any (TrySkippedMessageKeys logic essentially)
                self.skip_message_keys(header.pn)?;
                self.dh_ratchet(&header.dh_pub)?;
            }
        } else {
            // First message for Bob
            self.dh_ratchet(&header.dh_pub)?;
        }

        // 3. Skip messages in current chain
        self.skip_message_keys(header.n)?;

        // 4. Derive MK and decrypt
        let (ck_new, mk) = kdf_ck(
            self.ck_r
                .as_ref()
                .ok_or("Receiving chain not initialized")?,
        );
        self.ck_r = Some(ck_new);
        self.n_r += 1;

        decrypt_message(&mk, ciphertext, header)
    }

    fn skip_message_keys(&mut self, until: u32) -> Result<(), String> {
        if self.n_r + (SKIPPED_MESSAGE_KEYS_MAX as u32) < until {
            return Err("Too many skipped messages".to_string());
        }

        if let Some(ck_r) = self.ck_r {
            while self.n_r < until {
                let (ck_new, mk) = kdf_ck(&ck_r);
                self.ck_r = Some(ck_new);
                // Store mk
                if let Some(dhr) = self.dhr {
                    self.skipped_message_keys
                        .insert((dhr.as_bytes().to_vec(), self.n_r), mk);
                }
                self.n_r += 1;
            }
        }
        Ok(())
    }

    fn dh_ratchet(&mut self, new_dhr: &PublicKey) -> Result<(), String> {
        self.pn = self.n_s;
        self.n_s = 0;
        self.n_r = 0;
        self.dhr = Some(*new_dhr);

        // Ratchet Step:
        // 1. RK, DH(DHS, DHR) -> RK, CKr
        let dh1 = kdf_dh(&self.dhs, new_dhr);
        let (rk_new, ck_r_new) = kdf_rk(&self.rk, &dh1);
        self.rk = rk_new;
        self.ck_r = Some(ck_r_new);

        // 2. DHS = New Key
        self.dhs = StaticSecret::random_from_rng(rand::thread_rng());

        // 3. RK, DH(DHS, DHR) -> RK, CKs
        let dh2 = kdf_dh(&self.dhs, new_dhr);
        let (rk_new_2, ck_s_new) = kdf_rk(&self.rk, &dh2);
        self.rk = rk_new_2;
        self.ck_s = ck_s_new;

        Ok(())
    }
}

// --- Helpers ---

fn kdf_rk(rk: &[u8; 32], dh_out: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    // HKDF(salt=rk, ikm=dh_out, info="RatatoskrV1 Root") -> 64 bytes
    // Output: (Root Key, Chain Key)
    let hk = Hkdf::<Sha256>::new(Some(rk), dh_out);
    let mut okm = [0u8; 64];
    hk.expand(HKDF_INFO_ROOT, &mut okm)
        .expect("HKDF expand failed");

    let mut new_rk = [0u8; 32];
    let mut new_ck = [0u8; 32];
    new_rk.copy_from_slice(&okm[0..32]);
    new_ck.copy_from_slice(&okm[32..64]);
    (new_rk, new_ck)
}

fn kdf_ck(ck: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    // HMAC-SHA256(CK, b"\x01") -> New CK
    // HMAC-SHA256(CK, b"\x02") -> Message Key

    type HmacSha256 = Hmac<Sha256>;

    let mut mac = <HmacSha256 as Mac>::new_from_slice(ck).expect("HMAC init failed");
    mac.update(&[0x01]);
    let new_ck: [u8; 32] = mac.finalize().into_bytes().into();

    let mut mac2 = <HmacSha256 as Mac>::new_from_slice(ck).expect("HMAC init failed");
    mac2.update(&[0x02]);
    let mk: [u8; 32] = mac2.finalize().into_bytes().into();

    (new_ck, mk)
}

fn kdf_dh(priv_key: &StaticSecret, pub_key: &PublicKey) -> [u8; 32] {
    priv_key.diffie_hellman(pub_key).to_bytes()
}

fn encrypt_message(
    key: &[u8; 32],
    plaintext: &[u8],
    header: &RatchetHeader,
) -> Result<Vec<u8>, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Nonce should be unique per key. Since MK is unique per message, we can use a constant or deterministic nonce.
    // However, usually we use a random nonce or derive it.
    // For Signal, it's usually derived or fixed because the key is one-time use.
    // Let's use a zero nonce or derived from header.
    // Actually, AEAD requires unique (Key, Nonce) pair. Since Key is unique (MK), Nonce can be fixed 0.
    let nonce = Nonce::default(); // All zeros

    // Add header to Associated Data (AD) to bind it
    let header_bytes = serde_json::to_vec(header).map_err(|e| e.to_string())?;
    let payload = Payload {
        msg: plaintext,
        aad: &header_bytes,
    };

    cipher
        .encrypt(&nonce, payload)
        .map_err(|e| format!("Encryption error: {}", e))
}

fn decrypt_message(
    key: &[u8; 32],
    ciphertext: &[u8],
    header: &RatchetHeader,
) -> Result<Vec<u8>, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::default();

    let header_bytes = serde_json::to_vec(header).map_err(|e| e.to_string())?;
    let payload = Payload {
        msg: ciphertext,
        aad: &header_bytes,
    };

    cipher
        .decrypt(&nonce, payload)
        .map_err(|e| format!("Decryption error: {}", e))
}

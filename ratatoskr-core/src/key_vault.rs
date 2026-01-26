use bip39::{Language, Mnemonic};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs;
use std::path::Path;
use thiserror::Error;
use x25519_dalek::StaticSecret;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Mnemonic error: {0}")]
    Mnemonic(#[from] bip39::Error),
}

pub struct KeyVault {
    pub signing_key: SigningKey,
    pub dh_identity: StaticSecret,
}

impl KeyVault {
    /// Generates a new random Identity and returns it along with the Mnemonic phrase (12 words)
    pub fn generate_with_mnemonic() -> (Self, String) {
        // 1. Generate random entropy (128 bits = 16 bytes for 12 words)
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);

        let mnemonic =
            Mnemonic::from_entropy_in(Language::English, &entropy).expect("Entropy is valid");
        let phrase = mnemonic.to_string(); // In v2 to_string returns the phrase

        // 2. Derive key from mnemonic
        let vault = Self::recover(&phrase).expect("Generated mnemonic should be valid");

        (vault, phrase)
    }

    /// Recovers an Identity from a Mnemonic phrase
    pub fn recover(phrase: &str) -> Result<Self, VaultError> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase)?;
        let seed = mnemonic.to_seed(""); // No password for seed

        // BIP-39 seeds are 64 bytes.
        // First 32 bytes -> Ed25519 Signing Key
        let mut signing_bytes = [0u8; 32];
        signing_bytes.copy_from_slice(&seed[0..32]);
        let signing_key = SigningKey::from_bytes(&signing_bytes);

        // Second 32 bytes -> X25519 Identity Key
        let mut dh_bytes = [0u8; 32];
        dh_bytes.copy_from_slice(&seed[32..64]);
        let dh_identity = StaticSecret::from(dh_bytes);

        Ok(Self {
            signing_key,
            dh_identity,
        })
    }

    /// Legacy generation (random bytes, no mnemonic recovery possible unless saved)
    pub fn generate_random() -> Self {
        let mut signing_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut signing_bytes);
        let signing_key = SigningKey::from_bytes(&signing_bytes);

        let mut dh_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut dh_bytes);
        let dh_identity = StaticSecret::from(dh_bytes);

        Self {
            signing_key,
            dh_identity,
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), VaultError> {
        // We save 64 bytes: 32 bytes Ed25519 + 32 bytes X25519
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&self.signing_key.to_bytes());
        bytes.extend_from_slice(&self.dh_identity.to_bytes());
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, VaultError> {
        let bytes = fs::read(path)?;
        if bytes.len() != 64 {
            return Err(VaultError::Crypto(
                "Invalid key file length. Expected 64 bytes.".into(),
            ));
        }

        let signing_bytes: [u8; 32] = bytes[0..32].try_into().unwrap();
        let dh_bytes: [u8; 32] = bytes[32..64].try_into().unwrap();

        let signing_key = SigningKey::from_bytes(&signing_bytes);
        let dh_identity = StaticSecret::from(dh_bytes);

        Ok(Self {
            signing_key,
            dh_identity,
        })
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().as_bytes())
    }

    pub fn dh_public_key_hex(&self) -> String {
        hex::encode(x25519_dalek::PublicKey::from(&self.dh_identity).as_bytes())
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }
}

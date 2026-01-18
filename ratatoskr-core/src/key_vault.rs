use ed25519_dalek::SigningKey;
use std::fs;
use std::path::Path;
use thiserror::Error;
use rand::rngs::OsRng;
use rand::RngCore;
use bip39::{Mnemonic, Language};

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
    keypair: SigningKey,
}

impl KeyVault {
    /// Generates a new random Identity and returns it along with the Mnemonic phrase (12 words)
    pub fn generate_with_mnemonic() -> (Self, String) {
        // 1. Generate random entropy (128 bits = 16 bytes for 12 words)
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).expect("Entropy is valid");
        let phrase = mnemonic.to_string(); // In v2 to_string returns the phrase

        // 2. Derive key from mnemonic
        let vault = Self::recover(&phrase).expect("Generated mnemonic should be valid");
        
        (vault, phrase)
    }

    /// Recovers an Identity from a Mnemonic phrase
    pub fn recover(phrase: &str) -> Result<Self, VaultError> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase)?;
        let seed = mnemonic.to_seed(""); // No password for seed
        
        // We use the first 32 bytes of the seed as the Ed25519 secret key.
        // BIP-39 seeds are 64 bytes.
        let mut secret_bytes = [0u8; 32];
        secret_bytes.copy_from_slice(&seed[0..32]);
        
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        Ok(Self { keypair: signing_key })
    }

    /// Legacy generation (random bytes, no mnemonic recovery possible unless saved)
    pub fn generate_random() -> Self {
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        Self { keypair: signing_key }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), VaultError> {
        let bytes = self.keypair.to_bytes();
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, VaultError> {
        let bytes = fs::read(path)?;
        let secret_bytes: [u8; 32] = bytes.try_into()
            .map_err(|_| VaultError::Crypto("Invalid key length".into()))?;
        
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        Ok(Self { keypair: signing_key })
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.keypair.verifying_key().as_bytes())
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.keypair
    }
}
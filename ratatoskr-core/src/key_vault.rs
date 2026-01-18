use ed25519_dalek::SigningKey;
use std::fs;
use std::path::Path;
use thiserror::Error;
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Crypto error: {0}")]
    Crypto(String),
}

pub struct KeyVault {
    keypair: SigningKey,
}

impl KeyVault {
    /// Generates a new random Ed25519 identity
    pub fn generate() -> Self {
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        Self { keypair: signing_key }
    }

    /// Saves the secret key to a file (Unencrypted for now - Milestone 2 baseline)
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), VaultError> {
        let bytes = self.keypair.to_bytes();
        fs::write(path, bytes)?;
        Ok(())
    }

    /// Loads the secret key from a file
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

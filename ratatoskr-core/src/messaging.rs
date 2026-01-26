use crate::key_vault::KeyVault;
use crate::models::EncryptedMessage;
use crate::storage::Storage;
use crate::x3dh::{self, PreKeyBundle};
use ed25519_dalek::VerifyingKey;
use x25519_dalek::PublicKey;

pub struct MessagingService<'a> {
    storage: &'a Storage,
    vault: &'a KeyVault,
}

impl<'a> MessagingService<'a> {
    pub fn new(storage: &'a Storage, vault: &'a KeyVault) -> Self {
        Self { storage, vault }
    }

    /// Encrypts a message for a recipient.
    pub async fn encrypt_message(
        &self,
        recipient_did: &str,
        recipient_ed25519_key: Option<&VerifyingKey>, // Required if new session
        recipient_bundle: Option<&PreKeyBundle>,      // Required if new session
        plaintext: &[u8],
    ) -> Result<EncryptedMessage, String> {
        // 1. Load Session
        let session_opt = self
            .storage
            .load_session(recipient_did)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(mut session) = session_opt {
            // Existing Session
            let (header, ciphertext) = session.encrypt(plaintext).map_err(|e| e.to_string())?;

            // Save Session
            self.storage
                .save_session(recipient_did, &session)
                .await
                .map_err(|e| e.to_string())?;

            Ok(EncryptedMessage::Whisper { header, ciphertext })
        } else {
            // New Session - Needs Bundle
            let bundle = recipient_bundle.ok_or("No session and no PreKeyBundle provided")?;
            let recipient_vk =
                recipient_ed25519_key.ok_or("No session and no Recipient Identity Key provided")?;

            let result = x3dh::initialize_alice(
                &self.vault.dh_identity,
                PublicKey::from(&self.vault.dh_identity),
                bundle,
                recipient_vk,
                plaintext,
            )?;

            // Save Session
            self.storage
                .save_session(recipient_did, &result.session)
                .await
                .map_err(|e| e.to_string())?;

            Ok(EncryptedMessage::X3dhInit {
                sender_identity_key: PublicKey::from(&self.vault.dh_identity),
                ephemeral_key: result.ephemeral_key,
                header: result.initial_header,
                ciphertext: result.initial_ciphertext,
                used_spk: bundle.signed_prekey, // Tell Bob which SPK we used
                used_opk: result.used_opk,      // Tell Bob which OPK we used
            })
        }
    }

    /// Decrypts an incoming message
    pub async fn decrypt_message(
        &self,
        sender_did: &str,
        message: EncryptedMessage,
    ) -> Result<Vec<u8>, String> {
        match message {
            EncryptedMessage::Whisper { header, ciphertext } => {
                let mut session = self
                    .storage
                    .load_session(sender_did)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or("No session found for sender")?;

                let plaintext = session.decrypt(&header, &ciphertext)?;

                self.storage
                    .save_session(sender_did, &session)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(plaintext)
            }
            EncryptedMessage::X3dhInit {
                sender_identity_key,
                ephemeral_key,
                header,
                ciphertext,
                used_spk,
                used_opk,
            } => {
                // Bob receives init

                // 1. Fetch Secrets
                let spk_secret = self
                    .storage
                    .get_signed_prekey_secret(&used_spk)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or("Signed PreKey not found")?;

                let mut opk_secret = None;
                if let Some(opk_pub) = used_opk {
                    let secret = self
                        .storage
                        .get_onetime_prekey_secret(&opk_pub)
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or("One-time PreKey not found")?;
                    opk_secret = Some(secret);
                }

                // 2. Initialize
                let (session, plaintext) = x3dh::initialize_bob(
                    &self.vault.dh_identity,
                    &spk_secret,
                    opk_secret.as_ref(),
                    sender_identity_key,
                    ephemeral_key,
                    &header,
                    &ciphertext,
                )?;

                // 3. Save Session
                self.storage
                    .save_session(sender_did, &session)
                    .await
                    .map_err(|e| e.to_string())?;

                // 4. Delete consumed OPK
                if let Some(opk_pub) = used_opk {
                    self.storage
                        .delete_onetime_prekey(&opk_pub)
                        .await
                        .map_err(|e| e.to_string())?;
                }

                Ok(plaintext)
            }
        }
    }
}

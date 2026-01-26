use serde::{Deserialize, Serialize};
use x25519_dalek::PublicKey;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SosType {
    Medical,    // Medical assistance needed
    Evacuation, // Evacuation needed
    FoodWater,  // Food/Water
    Shelter,    // Shelter
    Violence,   // Threat of violence/shelling
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SosPayload {
    pub help_type: SosType,
    pub location: Option<GeoLocation>,
    pub description: String,
    pub timestamp: u64,
    // Important: We do NOT add UserID here for complete "Black Box" anonymity
}

// Encrypted packet that will travel through the network
#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedSosPacket {
    pub ephemeral_public_key: Vec<u8>, // Session public key (so the recipient can derive the secret)
    pub nonce: Vec<u8>,                // Random number for AES
    pub ciphertext: Vec<u8>,           // Encrypted SosPayload
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Direct,        // Standard human-to-human
    Ephemeral,     // Self-destructs after TTL
    Transactional, // Receipts, OTPs (auto-archive)
    Feed,          // News, logs (no notifications)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageStatus {
    Unread,
    ActionRequired,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub sender_did: String,
    pub recipient_did: String,
    pub msg_type: MessageType,
    pub status: MessageStatus,
    pub content: Vec<u8>, // Encrypted blob or Plaintext (depends on context, usually Plaintext in DB)
    pub timestamp: u64,
    pub ttl: Option<u64>,            // Optional expiry timestamp
    pub schema_id: String,           // For protocol extensibility
    pub reply_to_id: Option<String>, // ID of the message being replied to
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
}

// --- Protocol Models ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatchetHeader {
    pub dh_pub: PublicKey,
    pub n: u32,  // Number of the message in the sending chain
    pub pn: u32, // Number of the previous sending chain
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EncryptedMessage {
    /// Initial X3DH handshake message + first payload
    X3dhInit {
        sender_identity_key: PublicKey, // IK_A
        ephemeral_key: PublicKey,       // EK_A
        header: RatchetHeader,
        ciphertext: Vec<u8>,         // Initial message
        used_spk: PublicKey,         // The SPK used (so Bob knows which secret to pick)
        used_opk: Option<PublicKey>, // The OPK used (if any)
    },

    /// Subsequent Double Ratchet message
    Whisper {
        header: RatchetHeader,
        ciphertext: Vec<u8>,
    },
}

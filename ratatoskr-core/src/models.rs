use serde::{Deserialize, Serialize};

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
    pub content: Vec<u8>, // Encrypted blob
    pub timestamp: u64,
    pub ttl: Option<u64>,  // Optional expiry timestamp
    pub schema_id: String, // For protocol extensibility
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
}

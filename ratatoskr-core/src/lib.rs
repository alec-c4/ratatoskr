pub mod access_control;
pub mod crypto;
pub mod key_vault;
pub mod messaging;
pub mod models;
pub mod network;
pub mod protocol;
pub mod ratchet;
pub mod storage;
pub mod x3dh;

pub fn init() -> String {
    "Ratatoskr Core: Ready".to_string()
}

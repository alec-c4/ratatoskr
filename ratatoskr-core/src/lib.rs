pub mod crypto;
pub mod network;
pub mod models;
pub mod access_control;
pub mod key_vault;

pub fn init() -> String {
    "Ratatoskr Core: Ready".to_string()
}

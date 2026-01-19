pub mod access_control;
pub mod crypto;
pub mod key_vault;
pub mod models;
pub mod network;

pub fn init() -> String {
    "Ratatoskr Core: Ready".to_string()
}

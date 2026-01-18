use ratatoskr_core::models::{SosPayload, SosType, GeoLocation};
use ratatoskr_core::crypto::encrypt_sos_signal;
use ratatoskr_core::network::{build_swarm, run_network_node, NetworkCommand};
use ratatoskr_core::key_vault::KeyVault;
use std::time::{SystemTime, UNIX_EPOCH};
use libp2p::{identity, Multiaddr};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tauri::{State, Manager};
use std::path::PathBuf;
use std::fs;

// Application State: Stores the communication channel with the network thread
struct AppState {
    network_sender: Mutex<mpsc::Sender<NetworkCommand>>,
    identity_path: PathBuf,
}

#[tauri::command]
fn ping() -> String {
    ratatoskr_core::init()
}

#[tauri::command]
async fn get_identity(state: State<'_, AppState>) -> Result<Option<String>, String> {
    if state.identity_path.exists() {
        let vault = KeyVault::load_from_file(&state.identity_path)
            .map_err(|e| e.to_string())?;
        Ok(Some(vault.public_key_hex()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn create_identity(state: State<'_, AppState>) -> Result<(String, String), String> {
    if state.identity_path.exists() {
        return Err("Identity already exists".into());
    }
    
    let (vault, mnemonic) = KeyVault::generate_with_mnemonic();
    vault.save_to_file(&state.identity_path)
        .map_err(|e| e.to_string())?;
    
    Ok((vault.public_key_hex(), mnemonic))
}

#[tauri::command]
async fn recover_identity(state: State<'_, AppState>, phrase: String) -> Result<String, String> {
    if state.identity_path.exists() {
        return Err("Identity already exists. Delete it first to recover.".into());
    }

    let vault = KeyVault::recover(&phrase)
        .map_err(|e| format!("Invalid Mnemonic: {}", e))?;
    
    vault.save_to_file(&state.identity_path)
        .map_err(|e| e.to_string())?;
    
    Ok(vault.public_key_hex())
}

#[tauri::command]
async fn delete_identity(state: State<'_, AppState>) -> Result<(), String> {
    println!("DEBUG: Attempting to delete identity at {:?}", state.identity_path);
    if state.identity_path.exists() {
        fs::remove_file(&state.identity_path)
            .map_err(|e| {
                let err_msg = format!("Failed to delete key: {}", e);
                println!("DEBUG: {}", err_msg);
                err_msg
            })?;
        println!("DEBUG: Identity deleted successfully.");
    } else {
        println!("DEBUG: File does not exist, nothing to delete.");
    }
    Ok(())
}

#[tauri::command]
async fn export_backup(app_handle: tauri::AppHandle, content: String) -> Result<String, String> {
    // use tauri::path::BaseDirectory; // Removed unused import
    
    let download_dir = app_handle.path().download_dir()
        .map_err(|e| format!("Failed to resolve download dir: {}", e))?;
    
    let filename = format!("ratatoskr_backup_{}.txt", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
    let path = download_dir.join(filename);
    
    println!("DEBUG: Exporting backup to {:?}", path);
    
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write backup: {}", e))?;
        
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn send_sos(
    state: State<'_, AppState>,
    help_type: String, 
    lat: f64, 
    long: f64, 
    description: String
) -> Result<String, String> {
    
    // 1. Form the Payload (as before)
    let sos_type = match help_type.as_str() {
        "Medical" => SosType::Medical,
        "Evacuation" => SosType::Evacuation,
        "FoodWater" => SosType::FoodWater,
        "Shelter" => SosType::Shelter,
        _ => SosType::Violence,
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let payload = SosPayload {
        help_type: sos_type,
        location: Some(GeoLocation { latitude: lat, longitude: long, accuracy: 10.0 }),
        description,
        timestamp: now,
    };

    // 2. Encrypt (Organization Key)
    let trusted_key = [7u8; 32]; 
    let packet = encrypt_sos_signal(&payload, &trusted_key)
        .map_err(|e| format!("Encryption Error: {}", e))?;

    // 3. Serialize the packet for network transmission (to bytes)
    let packet_bytes = serde_json::to_vec(&packet)
        .map_err(|e| format!("Serialization Error: {}", e))?;

    // 4. Send the command to the network thread
    let sender = state.network_sender.lock().await;
    match sender.send(NetworkCommand::BroadcastSos(packet_bytes.clone())).await {
        Ok(_) => Ok(format!("SOS Broadcasted to Mesh Network! Size: {} bytes", packet_bytes.len())),
        Err(e) => Err(format!("Failed to send to network thread: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create channel: UI -> Network
    let (tx, rx) = mpsc::channel(32);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle();
            let config_dir = app_handle.path().app_config_dir().expect("Failed to get config dir");
            fs::create_dir_all(&config_dir).expect("Failed to create config dir");
            let identity_path = config_dir.join("identity.key");

            app.manage(AppState { 
                network_sender: Mutex::new(tx),
                identity_path: identity_path.clone(),
            });

            // Start P2P node in a separate background thread
            tauri::async_runtime::spawn(async move {
                // Load or generate temporary identity for the network
                let local_key = if identity_path.exists() {
                    match KeyVault::load_from_file(&identity_path) {
                        Ok(vault) => {
                            identity::Keypair::ed25519_from_bytes(vault.signing_key().to_bytes())
                                .expect("Failed to convert key")
                        },
                        Err(_) => identity::Keypair::generate_ed25519()
                    }
                } else {
                    identity::Keypair::generate_ed25519()
                };

                match build_swarm(local_key).await {
                    Ok(swarm) => {
                        println!("Desktop P2P Node Started");
                        
                        // Better approach: Dial directly on swarm before running loop
                        let mut active_swarm = swarm;
                        if let Ok(addr) = "/ip4/127.0.0.1/tcp/4001".parse::<Multiaddr>() {
                             println!("Bootstrapping: Connecting to local relay...");
                             let _ = active_swarm.dial(addr);
                        }

                        if let Err(e) = run_network_node(active_swarm, rx).await {
                            eprintln!("Network node crashed: {}", e);
                        }
                    },
                    Err(e) => eprintln!("Failed to build swarm: {}", e),
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ping, send_sos, get_identity, create_identity, recover_identity, delete_identity, export_backup])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

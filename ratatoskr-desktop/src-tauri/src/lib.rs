use ed25519_dalek::VerifyingKey;
use libp2p::{identity, Multiaddr};
use rand::rngs::OsRng;
use ratatoskr_core::crypto::encrypt_sos_signal;
use ratatoskr_core::key_vault::KeyVault;
use ratatoskr_core::models::{GeoLocation, SosPayload, SosType};
use ratatoskr_core::network::{build_swarm, run_network_node, NetworkCommand};
use ratatoskr_core::storage::Storage;
use ratatoskr_core::x3dh::PreKeyBundle;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager, State};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use x25519_dalek::StaticSecret;

// Application State: Stores the communication channel with the network thread
struct AppState {
    network_sender: Mutex<mpsc::Sender<NetworkCommand>>,
    identity_path: PathBuf,
    storage: Arc<Storage>,
}

#[tauri::command]
fn ping() -> String {
    ratatoskr_core::init()
}

#[tauri::command]
async fn get_messages(
    state: State<'_, AppState>,
    did: String,
) -> Result<Vec<ratatoskr_core::models::ChatMessage>, String> {
    state
        .storage
        .list_messages(&did)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_message(
    state: State<'_, AppState>,
    recipient_did: String,
    content: String,
    msg_type_str: String,
    reply_to_id: Option<String>,
) -> Result<(), String> {
    use ratatoskr_core::models::{ChatMessage, MessageStatus, MessageType};

    let msg_type = match msg_type_str.as_str() {
        "Ephemeral" => MessageType::Ephemeral,
        "Transactional" => MessageType::Transactional,
        "Feed" => MessageType::Feed,
        _ => MessageType::Direct,
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Calculate TTL (60 seconds for Ephemeral messages for testing)
    let ttl = if matches!(msg_type, MessageType::Ephemeral) {
        Some(now + 60)
    } else {
        None
    };

    // 0. Get our own DID and Vault
    let vault = if state.identity_path.exists() {
        KeyVault::load_from_file(&state.identity_path)
            .map_err(|e| format!("Failed to load identity: {}", e))?
    } else {
        return Err("No identity found".into());
    };
    let sender_did = vault.public_key_hex();

    // 1. Encrypt message using MessagingService
    let service = ratatoskr_core::messaging::MessagingService::new(&state.storage, &vault);

    // Try to load bundle and keys if session doesn't exist
    let bundle_opt = state
        .storage
        .get_bundle(&recipient_did)
        .await
        .ok()
        .flatten();
    let vk_opt = if let Ok(bytes) = hex::decode(&recipient_did) {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            VerifyingKey::from_bytes(&arr).ok()
        } else {
            None
        }
    } else {
        None
    };

    let encrypted_result = service
        .encrypt_message(
            &recipient_did,
            vk_opt.as_ref(),
            bundle_opt.as_ref(),
            content.as_bytes(),
        )
        .await;

    let encrypted_msg = match encrypted_result {
        Ok(msg) => msg,
        Err(e) => {
            // If encryption failed, it might be due to missing session/bundle.
            // Trigger discovery and fail gracefully.
            let sender: tokio::sync::MutexGuard<mpsc::Sender<NetworkCommand>> =
                state.network_sender.lock().await;
            let _ = sender
                .send(NetworkCommand::GetBundle(recipient_did.clone()))
                .await;

            return Err(format!("Secure session not established. Discovery initiated. Please retry in a few seconds. (Error: {})", e));
        }
    };

    // 2. Save plaintext to local DB for UI
    let msg = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        sender_did: sender_did.clone(),
        recipient_did: recipient_did.clone(),
        msg_type,
        status: MessageStatus::Done, // Outgoing messages marked as done locally
        content: content.into_bytes(),
        timestamp: now,
        ttl,
        schema_id: "text".to_string(),
        reply_to_id,
    };

    state
        .storage
        .save_message(&msg)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Send to Network
    let sender: tokio::sync::MutexGuard<mpsc::Sender<NetworkCommand>> =
        state.network_sender.lock().await;
    sender
        .send(NetworkCommand::SendDirectMessage {
            recipient_did,
            sender_did,
            message: Box::new(encrypted_msg),
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn update_message_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
) -> Result<(), String> {
    use ratatoskr_core::models::MessageStatus;
    let status_enum = match status.as_str() {
        "Unread" => MessageStatus::Unread,
        "ActionRequired" => MessageStatus::ActionRequired,
        _ => MessageStatus::Done,
    };

    state
        .storage
        .update_message_status(&id, status_enum)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn add_contact(state: State<'_, AppState>, did: String, alias: String) -> Result<(), String> {
    state
        .storage
        .add_contact(&did, &alias)
        .await
        .map_err(|e| e.to_string())?;

    // Try to find the peer's bundle immediately
    let sender: tokio::sync::MutexGuard<mpsc::Sender<NetworkCommand>> =
        state.network_sender.lock().await;
    let _ = sender.send(NetworkCommand::GetBundle(did)).await;

    Ok(())
}

#[tauri::command]
async fn update_contact(
    state: State<'_, AppState>,
    did: String,
    alias: String,
) -> Result<(), String> {
    state
        .storage
        .update_contact(&did, &alias)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn delete_contact(state: State<'_, AppState>, did: String) -> Result<(), String> {
    state
        .storage
        .delete_contact(&did)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<(String, Option<String>)>, String> {
    let contacts = state
        .storage
        .list_contacts()
        .await
        .map_err(|e| e.to_string())?;
    Ok(contacts)
}

#[tauri::command]
async fn get_identity(state: State<'_, AppState>) -> Result<Option<String>, String> {
    if state.identity_path.exists() {
        let vault = KeyVault::load_from_file(&state.identity_path).map_err(|e| e.to_string())?;
        Ok(Some(vault.public_key_hex()))
    } else {
        Ok(None)
    }
}

async fn publish_my_bundle(state: &AppState, vault: &KeyVault) -> Result<(), String> {
    // 1. Generate keys
    let spk_secret = StaticSecret::random_from_rng(OsRng);
    let opk_secret = StaticSecret::random_from_rng(OsRng);

    // 2. Save secrets to storage
    state
        .storage
        .save_signed_prekey(&spk_secret)
        .await
        .map_err(|e| e.to_string())?;
    state
        .storage
        .save_onetime_prekey(&opk_secret)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Create bundle
    let bundle = PreKeyBundle::new(
        &vault.dh_identity,
        &vault.signing_key,
        &spk_secret,
        Some(&opk_secret),
    );

    // 4. Send to network
    let sender = state.network_sender.lock().await;
    sender
        .send(NetworkCommand::PublishBundle {
            did: vault.public_key_hex(),
            bundle,
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn create_identity(
    state: State<'_, AppState>,
    nickname: String,
) -> Result<(String, String), String> {
    if state.identity_path.exists() {
        return Err("Identity already exists".into());
    }

    let (vault, mnemonic) = KeyVault::generate_with_mnemonic();
    vault
        .save_to_file(&state.identity_path)
        .map_err(|e| e.to_string())?;

    // Save profile info
    let profile_path = state.identity_path.parent().unwrap().join("profile.json");
    let profile_data = serde_json::json!({
        "nickname": nickname,
        "did": vault.public_key_hex()
    });
    fs::write(
        profile_path,
        serde_json::to_string_pretty(&profile_data).unwrap(),
    )
    .map_err(|e| format!("Failed to save profile: {}", e))?;

    // Publish bundle to network
    publish_my_bundle(&state, &vault).await?;

    // Update Network Thread Identity
    let sender = state.network_sender.lock().await;
    sender
        .send(NetworkCommand::SetIdentity(vault.public_key_hex()))
        .await
        .map_err(|e| e.to_string())?;

    Ok((vault.public_key_hex(), mnemonic))
}

#[tauri::command]
async fn get_profile_name(state: State<'_, AppState>) -> Result<String, String> {
    let profile_path = state.identity_path.parent().unwrap().join("profile.json");
    if profile_path.exists() {
        let content = fs::read_to_string(profile_path).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(json["nickname"].as_str().unwrap_or("Unknown").to_string())
    } else {
        Ok("Anonymous".to_string())
    }
}

#[tauri::command]
async fn recover_identity(state: State<'_, AppState>, phrase: String) -> Result<String, String> {
    if state.identity_path.exists() {
        return Err("Identity already exists. Delete it first to recover.".into());
    }

    let vault = KeyVault::recover(&phrase).map_err(|e| format!("Invalid Mnemonic: {}", e))?;

    vault
        .save_to_file(&state.identity_path)
        .map_err(|e| e.to_string())?;

    // Publish bundle to network
    publish_my_bundle(&state, &vault).await?;

    // Update Network Thread Identity
    let sender = state.network_sender.lock().await;
    sender
        .send(NetworkCommand::SetIdentity(vault.public_key_hex()))
        .await
        .map_err(|e| e.to_string())?;

    Ok(vault.public_key_hex())
}

#[tauri::command]
async fn delete_identity(state: State<'_, AppState>) -> Result<(), String> {
    println!(
        "DEBUG: Attempting to delete identity at {:?}",
        state.identity_path
    );
    if state.identity_path.exists() {
        fs::remove_file(&state.identity_path).map_err(|e| {
            let err_msg = format!("Failed to delete key: {}", e);
            println!("DEBUG: {}", err_msg);
            err_msg
        })?;
        println!("DEBUG: Identity deleted successfully.");
        // Reset Network Identity
        let sender = state.network_sender.lock().await;
        let _ = sender
            .send(NetworkCommand::SetIdentity("anonymous".to_string()))
            .await;
    } else {
        println!("DEBUG: File does not exist, nothing to delete.");
    }
    Ok(())
}

#[tauri::command]
async fn export_backup(app_handle: tauri::AppHandle, content: String) -> Result<String, String> {
    let download_dir = app_handle
        .path()
        .download_dir()
        .map_err(|e| format!("Failed to resolve download dir: {}", e))?;

    let filename = format!(
        "ratatoskr_backup_{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let path = download_dir.join(filename);

    println!("DEBUG: Exporting backup to {:?}", path);

    fs::write(&path, content).map_err(|e| format!("Failed to write backup: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn send_sos(
    state: State<'_, AppState>,
    help_type: String,
    lat: f64,
    long: f64,
    description: String,
) -> Result<String, String> {
    let sos_type = match help_type.as_str() {
        "Medical" => SosType::Medical,
        "Evacuation" => SosType::Evacuation,
        "FoodWater" => SosType::FoodWater,
        "Shelter" => SosType::Shelter,
        _ => SosType::Violence,
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let payload = SosPayload {
        help_type: sos_type,
        location: Some(GeoLocation {
            latitude: lat,
            longitude: long,
            accuracy: 10.0,
        }),
        description,
        timestamp: now,
    };

    let trusted_key = [7u8; 32];
    let packet = encrypt_sos_signal(&payload, &trusted_key)
        .map_err(|e| format!("Encryption Error: {}", e))?;

    let packet_bytes =
        serde_json::to_vec(&packet).map_err(|e| format!("Serialization Error: {}", e))?;

    let sender: tokio::sync::MutexGuard<mpsc::Sender<NetworkCommand>> =
        state.network_sender.lock().await;
    match sender
        .send(NetworkCommand::BroadcastSos(packet_bytes.clone()))
        .await
    {
        Ok(_) => Ok(format!(
            "SOS Broadcasted to Mesh Network! Size: {} bytes",
            packet_bytes.len()
        )),
        Err(e) => Err(format!("Failed to send to network thread: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = mpsc::channel(32);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle();
            let config_dir = if let Ok(custom_path) = std::env::var("RATATOSKR_CONFIG_DIR") {
                println!("Using custom config dir: {}", custom_path);
                PathBuf::from(custom_path)
            } else {
                app_handle
                    .path()
                    .app_config_dir()
                    .expect("Failed to get config dir")
            };

            fs::create_dir_all(&config_dir).expect("Failed to create config dir");
            let identity_path = config_dir.join("identity.key");
            let db_path = config_dir.join("ratatoskr.db");

            let storage = tauri::async_runtime::block_on(async {
                Storage::init(&db_path).await.expect("Failed to init DB")
            });
            let storage_arc = Arc::new(storage);

            app.manage(AppState {
                network_sender: Mutex::new(tx),
                identity_path: identity_path.clone(),
                storage: storage_arc.clone(),
            });

            let (event_tx, mut event_rx) = mpsc::channel(32);
            let app_handle_clone = app_handle.clone();
            let storage_for_events = storage_arc.clone();
            let identity_path_clone = identity_path.clone();

            tauri::async_runtime::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    match event {
                        ratatoskr_core::network::NetworkEvent::PeerCountUpdated(count) => {
                            let _ = app_handle_clone.emit("peer-count-update", count);
                        }
                        ratatoskr_core::network::NetworkEvent::MessageReceived {
                            topic,
                            payload: _,
                            sender: _,
                        } => {
                            println!("UI: Received gossip message on topic {}", topic);
                            if topic == "ratatoskr-sos" {
                                // Handle SOS
                            }
                        }
                        ratatoskr_core::network::NetworkEvent::DirectMessageReceived {
                            sender_did,
                            message,
                        } => {
                            println!("UI: Received direct message from {}", sender_did);

                            // 1. Initialize MessagingService
                            if identity_path_clone.exists() {
                                if let Ok(vault) = KeyVault::load_from_file(&identity_path_clone) {
                                    let service = ratatoskr_core::messaging::MessagingService::new(
                                        &storage_for_events,
                                        &vault,
                                    );

                                    // 2. Decrypt
                                    match service.decrypt_message(&sender_did, *message).await {
                                        Ok(plaintext) => {
                                            println!("UI: Decrypted DM from {}", sender_did);
                                            // 3. Save to DB
                                            let msg = ratatoskr_core::models::ChatMessage {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                sender_did: sender_did.clone(),
                                                recipient_did: "me".to_string(),
                                                msg_type:
                                                    ratatoskr_core::models::MessageType::Direct,
                                                status:
                                                    ratatoskr_core::models::MessageStatus::Unread,
                                                content: plaintext,
                                                timestamp: SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_secs(),
                                                ttl: None,
                                                schema_id: "text".to_string(),
                                                reply_to_id: None,
                                            };

                                            if let Err(e) =
                                                storage_for_events.save_message(&msg).await
                                            {
                                                eprintln!(
                                                    "Failed to save decrypted message: {}",
                                                    e
                                                );
                                            }
                                            let _ = app_handle_clone.emit("msg-received", msg);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to decrypt direct message: {}", e)
                                        }
                                    }
                                }
                            }
                        }
                        ratatoskr_core::network::NetworkEvent::BundleFound { did, bundle } => {
                            println!("UI: Storing PreKeyBundle for {}", did);
                            if let Err(e) = storage_for_events.save_bundle(&did, &bundle).await {
                                eprintln!("Failed to save bundle: {}", e);
                            }
                        }
                        ratatoskr_core::network::NetworkEvent::MessageStored => {
                            println!("UI: Message successfully stored in mailbox");
                        }
                        ratatoskr_core::network::NetworkEvent::MailboxMessages(msgs) => {
                            println!("UI: Retrieved {} messages from mailbox", msgs.len());
                            // TODO: Decrypt and save messages
                        }
                        ratatoskr_core::network::NetworkEvent::MailboxError(e) => {
                            eprintln!("UI: Mailbox Error: {}", e);
                        }
                    }
                }
            });

            let storage_gc = storage_arc.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    match storage_gc.cleanup_expired_messages().await {
                        Ok(count) if count > 0 => {
                            println!("GC: Deleted {} expired messages", count)
                        }
                        Err(e) => eprintln!("GC Error: {}", e),
                        _ => {}
                    }
                }
            });

            tauri::async_runtime::spawn(async move {
                let (local_key, local_did) = if identity_path.exists() {
                    match KeyVault::load_from_file(&identity_path) {
                        Ok(vault) => {
                            let key = identity::Keypair::ed25519_from_bytes(
                                vault.signing_key().to_bytes(),
                            )
                            .expect("Failed to convert key");
                            (key, vault.public_key_hex())
                        }
                        Err(_) => (
                            identity::Keypair::generate_ed25519(),
                            "anonymous".to_string(),
                        ),
                    }
                } else {
                    (
                        identity::Keypair::generate_ed25519(),
                        "anonymous".to_string(),
                    )
                };

                match build_swarm(local_key).await {
                    Ok(swarm) => {
                        println!("Desktop P2P Node Started as {}", local_did);
                        let mut active_swarm = swarm;
                        if let Ok(addr) = "/ip4/127.0.0.1/tcp/4001".parse::<Multiaddr>() {
                            println!("Bootstrapping: Connecting to local relay...");
                            let _ = active_swarm.dial(addr);
                        }
                        if let Err(e) =
                            run_network_node(active_swarm, local_did, rx, event_tx).await
                        {
                            eprintln!("Network node crashed: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to build swarm: {}", e),
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            send_sos,
            get_identity,
            get_profile_name,
            create_identity,
            recover_identity,
            delete_identity,
            export_backup,
            add_contact,
            update_contact,
            delete_contact,
            get_contacts,
            get_messages,
            send_message,
            update_message_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

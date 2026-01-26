use libp2p::futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    gossipsub, kad, mdns, noise, swarm::NetworkBehaviour, yamux, Multiaddr, Swarm, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io;
use tokio::sync::mpsc;

// Commands that the UI can send to the Network
#[derive(Debug)]
pub enum NetworkCommand {
    BroadcastSos(Vec<u8>),  // Broadcast an encrypted SOS packet
    Dial(Multiaddr),        // Connect to a specific peer manually
    StartProviding(String), // Announce our ID to the DHT
    FindPeer(String),       // Find peer address by ID
    SendDirectMessage {
        recipient_did: String,
        sender_did: String,
        message: Box<crate::models::EncryptedMessage>,
    },
    PublishBundle {
        did: String,
        bundle: crate::x3dh::PreKeyBundle,
    },
    GetBundle(String),
}

// Events that the Network sends to the UI
#[derive(Debug)]
pub enum NetworkEvent {
    MessageReceived {
        topic: String,
        payload: Vec<u8>,
        sender: String,
    },
    PeerCountUpdated(usize),
    DirectMessageReceived {
        sender_did: String,
        message: Box<crate::models::EncryptedMessage>,
    },
    BundleFound {
        did: String,
        bundle: crate::x3dh::PreKeyBundle,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectMessagePayload {
    pub recipient_did: String,
    pub sender_did: String,
    pub message: crate::models::EncryptedMessage,
}

// Network behavior
#[derive(NetworkBehaviour)]
pub struct RatatoskrBehavior {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

pub async fn build_swarm(
    local_key: Keypair,
) -> Result<Swarm<RatatoskrBehavior>, Box<dyn std::error::Error + Send + Sync>> {
    let local_peer_id = local_key.public().to_peer_id();
    println!("Local Peer ID: {}", local_peer_id);

    let swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // Gossipsub Config
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(1))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .mesh_outbound_min(0)
                .mesh_n_low(1)
                .mesh_n(2)
                .mesh_n_high(4)
                .build()
                .map_err(io::Error::other)?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .map_err(io::Error::other)?;

            // mDNS Config
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;

            // Kademlia DHT Config
            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            let kademlia = kad::Behaviour::new(key.public().to_peer_id(), store);

            Ok(RatatoskrBehavior {
                gossipsub,
                mdns,
                kademlia,
            })
        })?
        .build();

    Ok(swarm)
}

// Main loop of the network node
pub async fn run_network_node(
    mut swarm: Swarm<RatatoskrBehavior>,
    local_did: String,
    mut command_receiver: mpsc::Receiver<NetworkCommand>,
    event_sender: mpsc::Sender<NetworkEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Subscribe to the SOS channel
    let sos_topic = gossipsub::IdentTopic::new("ratatoskr-sos");
    swarm.behaviour_mut().gossipsub.subscribe(&sos_topic)?;

    // Subscribe to Direct Messages channel
    let dm_topic = gossipsub::IdentTopic::new("ratatoskr-direct");
    swarm.behaviour_mut().gossipsub.subscribe(&dm_topic)?;

    // Set Kademlia to Server mode (auto-update routing table)
    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    // Listen on any available port
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut stats_interval = tokio::time::interval(Duration::from_secs(5));
    let mut last_peer_count = 0;

    loop {
        tokio::select! {
            _ = stats_interval.tick() => {
                let current_peers = swarm.network_info().num_peers();
                if current_peers != last_peer_count {
                    last_peer_count = current_peers;
                    let _ = event_sender.send(NetworkEvent::PeerCountUpdated(current_peers)).await;
                }
            },
            // 1. Network Events
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Client listening on {:?}", address);
                },
                SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        println!("Found peer via mDNS: {:?} at {:?}", peer_id, multiaddr);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr.clone());
                        // Explicitly dial to establish connection for GossipSub
                        match swarm.dial(multiaddr.clone()) {
                            Ok(_) => println!("Dialing {:?}...", multiaddr),
                            Err(e) => println!("Failed to dial {:?}: {:?}", multiaddr, e),
                        }
                    }
                },
                SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                    result: kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { providers, key, .. })),
                    ..
                })) => {
                    println!("DHT: Found providers for key {:?}: {:?}", key, providers);
                },
                SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                    result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(record))),
                    ..
                })) => {
                    let key_str = String::from_utf8_lossy(record.record.key.as_ref());
                    if key_str.starts_with("bundle:") {
                        let did = key_str.trim_start_matches("bundle:").to_string();
                        if let Ok(bundle) = serde_json::from_slice::<crate::x3dh::PreKeyBundle>(&record.record.value) {
                             println!("DHT: Found PreKeyBundle for DID: {}", did);
                             let _ = event_sender.send(NetworkEvent::BundleFound {
                                 did,
                                 bundle,
                             }).await;
                        }
                    }
                },
                SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic })) => {
                    println!("Peer {:?} subscribed to {:?}", peer_id, topic);
                },
                SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: _id,
                        message,
                    },
                )) => {
                    println!("🚨 RECEIVED MESSAGE from {:?}", peer_id);
                    println!("   Topic: {}", message.topic);

                    if message.topic.as_str() == "ratatoskr-direct" {
                        if let Ok(payload) = serde_json::from_slice::<DirectMessagePayload>(&message.data) {
                            if payload.recipient_did == local_did {
                                println!("   Direct Message for ME from {}", payload.sender_did);
                                let _ = event_sender.send(NetworkEvent::DirectMessageReceived {
                                    sender_did: payload.sender_did,
                                    message: Box::new(payload.message),
                                }).await;
                            }
                        }
                    } else {
                        let _ = event_sender.send(NetworkEvent::MessageReceived {
                            topic: message.topic.to_string(),
                            payload: message.data,
                            sender: peer_id.to_string(),
                        }).await;
                    }
                },
                _ => {}
            },

            // 2. Commands from UI
            command = command_receiver.recv() => match command {
                Some(NetworkCommand::BroadcastSos(data)) => {
                    println!("Network: Broadcasting SOS packet ({} bytes)", data.len());
                    // Retry logic for SOS (critical message)
                    for i in 0..5 {
                        match swarm.behaviour_mut().gossipsub.publish(sos_topic.clone(), data.clone()) {
                            Ok(_) => {
                                println!("SOS Broadcast successful.");
                                break;
                            },
                            Err(gossipsub::PublishError::InsufficientPeers) => {
                                println!("Attempt {}: Insufficient peers, retrying in 2s...", i + 1);
                                tokio::time::sleep(Duration::from_secs(2)).await;
                            },
                            Err(e) => {
                                println!("Publish error: {:?}", e);
                                break;
                            }
                        }
                    }
                },
                Some(NetworkCommand::Dial(addr)) => {
                    println!("Network: Dialing {}...", addr);
                    if let Err(e) = swarm.dial(addr) {
                        println!("Dial error: {:?}", e);
                    }
                },
                Some(NetworkCommand::StartProviding(key_str)) => {
                    let key = kad::RecordKey::new(&key_str);
                    println!("DHT: Announcing provider for key: {}", key_str);
                    if let Err(e) = swarm.behaviour_mut().kademlia.start_providing(key) {
                         println!("DHT Provide Error: {:?}", e);
                    }
                },
                Some(NetworkCommand::FindPeer(key_str)) => {
                    let key = kad::RecordKey::new(&key_str);
                    println!("DHT: Searching for key: {}", key_str);
                    swarm.behaviour_mut().kademlia.get_providers(key);
                },
                Some(NetworkCommand::SendDirectMessage { recipient_did, sender_did, message }) => {
                    println!("Network: Sending direct message to {}", recipient_did);
                    let payload = DirectMessagePayload {
                        recipient_did,
                        sender_did,
                        message: *message,
                    };
                    if let Ok(data) = serde_json::to_vec(&payload) {
                        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(dm_topic.clone(), data) {
                            println!("Direct Message Publish Error: {:?}", e);
                        }
                    }
                },
                Some(NetworkCommand::PublishBundle { did, bundle }) => {
                    println!("Network: Publishing PreKeyBundle for DID: {}", did);
                    if let Ok(data) = serde_json::to_vec(&bundle) {
                        let record = kad::Record {
                            key: kad::RecordKey::new(&format!("bundle:{}", did)),
                            value: data,
                            publisher: None,
                            expires: None,
                        };
                        if let Err(e) = swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One) {
                            println!("DHT PutRecord Error: {:?}", e);
                        }
                    }
                },
                Some(NetworkCommand::GetBundle(did)) => {
                    println!("Network: Searching for PreKeyBundle for DID: {}", did);
                    let key = kad::RecordKey::new(&format!("bundle:{}", did));
                    swarm.behaviour_mut().kademlia.get_record(key);
                }
                None => break, // Channel closed, exiting
            }
        }
    }
    Ok(())
}

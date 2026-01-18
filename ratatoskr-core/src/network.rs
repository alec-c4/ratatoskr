use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, yamux, Swarm, SwarmBuilder, Multiaddr
};
use libp2p::identity::Keypair;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io;
use tokio::sync::mpsc;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;

// Commands that the UI can send to the Network
#[derive(Debug)]
pub enum NetworkCommand {
    BroadcastSos(Vec<u8>), // Broadcast an encrypted SOS packet
    Dial(Multiaddr),       // Connect to a specific peer manually
}

// Network behavior
#[derive(NetworkBehaviour)]
pub struct RatatoskrBehavior {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
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
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; 

            // mDNS Config
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(), 
                key.public().to_peer_id()
            )?;

            Ok(RatatoskrBehavior { gossipsub, mdns })
        })?
        .build();

    Ok(swarm)
}

// Main loop of the network node
pub async fn run_network_node(
    mut swarm: Swarm<RatatoskrBehavior>,
    mut command_receiver: mpsc::Receiver<NetworkCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // Subscribe to the SOS channel
    let sos_topic = gossipsub::IdentTopic::new("ratatoskr-sos");
    swarm.behaviour_mut().gossipsub.subscribe(&sos_topic)?;

    // Listen on any available port
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        tokio::select! {
            // 1. Network Events
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Client listening on {:?}", address);
                },
                SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        println!("Found peer via mDNS: {:?} at {:?}", peer_id, multiaddr);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        swarm.add_peer_address(peer_id, multiaddr); 
                    }
                },
                SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic })) => {
                    println!("Peer {:?} subscribed to {:?}", peer_id, topic);
                },
                _ => {}
            },

            // 2. Commands from UI
            command = command_receiver.recv() => match command {
                Some(NetworkCommand::BroadcastSos(data)) => {
                    println!("Network: Broadcasting SOS packet ({} bytes)", data.len());
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(sos_topic.clone(), data) {
                        println!("Publish error: {:?}", e);
                    }
                },
                Some(NetworkCommand::Dial(addr)) => {
                    println!("Network: Dialing {}...", addr);
                    if let Err(e) = swarm.dial(addr) {
                        println!("Dial error: {:?}", e);
                    }
                },
                None => break, // Channel closed, exiting
            }
        }
    }
    Ok(())
}

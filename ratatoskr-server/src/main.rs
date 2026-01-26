use env_logger::Env;
use libp2p::identity;
use libp2p::swarm::SwarmEvent;
use libp2p::{futures::StreamExt, gossipsub, kad};
use ratatoskr_core::network::{build_swarm, RatatoskrBehaviorEvent};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // 1. Generate Identity for the Server
    let local_key = identity::Keypair::generate_ed25519();

    // 2. Create P2P Swarm
    let mut swarm = build_swarm(local_key).await?;

    // 3. Subscribe to channels
    let sos_topic = gossipsub::IdentTopic::new("ratatoskr-sos");
    swarm.behaviour_mut().gossipsub.subscribe(&sos_topic)?;

    let dm_topic = gossipsub::IdentTopic::new("ratatoskr-direct");
    swarm.behaviour_mut().gossipsub.subscribe(&dm_topic)?;

    // Set Server Mode for DHT
    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    // 4. Listen on port 4001 (standard for IPFS/libp2p)
    swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?)?;

    println!("✅ Ratatoskr Relay Node Started.");
    println!("📡 Listening for SOS signals on 'ratatoskr-sos'...");

    // 5. Event Loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("👂 Listening on {:?}", address);
            }
            SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Mdns(event)) => {
                println!("🔍 mDNS Discovery: {:?}", event);
                // Note: In core/network.rs we already handle adding mDNS peers to Kademlia automatically
                // inside run_network_node logic, but here we are running a raw loop.
                // We should replicate that logic or reuse run_network_node if possible.
                // For now, let's manually add them to DHT here too.
                if let libp2p::mdns::Event::Discovered(list) = event {
                    for (peer_id, multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, multiaddr);
                    }
                }
            }
            SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Gossipsub(
                gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                },
            )) => {
                println!("🚨 RECEIVED MESSAGE from {:?}", peer_id);
                println!("   Message ID: {}", id);
                println!("   Topic: {:?}", message.topic);
                println!("   Payload Size: {} bytes", message.data.len());
                // Future: Save to DB (blind storage)
            }
            SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Kademlia(event)) => {
                println!("🌐 DHT Event: {:?}", event);
            }
            _ => {}
        }
    }
}

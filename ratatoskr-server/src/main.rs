use ratatoskr_core::network::{build_swarm, RatatoskrBehaviorEvent};
use libp2p::{gossipsub, futures::StreamExt};
use libp2p::swarm::SwarmEvent;
use libp2p::identity;
use std::error::Error;
use env_logger::Env;

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

    // 4. Listen on port 4001 (standard for IPFS/libp2p)
    swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?)?;

    println!("✅ Ratatoskr Relay Node Started.");
    println!("📡 Listening for SOS signals on 'ratatoskr-sos'...");

    // 5. Event Loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("👂 Listening on {:?}", address);
            },
            SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Mdns(event)) => {
                println!("🔍 mDNS Discovery: {:?}", event);
            },
            SwarmEvent::Behaviour(RatatoskrBehaviorEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                println!("🚨 RECEIVED MESSAGE from {:?}", peer_id);
                println!("   Message ID: {}", id);
                println!("   Topic: {:?}", message.topic);
                println!("   Payload Size: {} bytes", message.data.len());
                // Future: Save to DB (blind storage)
            },
            _ => {}
        }
    }
}
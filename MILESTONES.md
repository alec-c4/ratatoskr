# Ratatoskr Project Milestones

## 🏁 Milestone 1: The "Beacon" (v0.1.0)
**Focus:** Core Architecture & Emergency Broadcasting.
The minimum viable product demonstrating that the P2P stack works and the "Black Box" crypto is functional.

- **Goals:**
    - [x] Rust Core + Tauri + Svelte integration.
    - [x] Basic P2P networking (GossipSub + mDNS).
    - [x] SOS Protocol implementation (ECIES).
    - [x] Tactical UI for sending emergency signals.
- **Success Criteria:** A user can launch the app, press "Medical SOS", and a relay node on the same network receives and decrypts the packet log.

## 🪪 Milestone 2: Identity & Discovery (v0.2.0)
**Focus:** Self-Sovereign Identity (SSI) & Routing.
Turning the app from a signal emitter into a user-aware client.

- **Goals:**
    - [x] Key management (Ed25519) & secure local storage (KeyVault).
    - [x] DID (Decentralized Identifier) generation.
    - [x] "First Run" User Experience (Registration).
    - [ ] DHT (Kademlia) integration to find peers by ID, not just IP.
- **Success Criteria:** User A can search for User B by their Public Key/DID and establish a direct connection.

## 💬 Milestone 3: Secure Conversations (v0.3.0)
**Focus:** End-to-End Encrypted Chat & Offline Delivery.
A fully functional messenger.

- **Goals:**
    - [ ] Signal Protocol (Double Ratchet) implementation.
    - [ ] SQLite integration for message history.
    - [ ] "Blind Mailbox" logic for offline message storage.
    - [ ] Basic text chat UI.
- **Success Criteria:** User A sends a message to User B while B is offline. User B comes online later and receives the message.

## 🛡️ Milestone 4: The "Bunker" Update (v0.4.0)
**Focus:** Extreme Security & Resilience.
Features for hostile environments.

- **Goals:**
    - [ ] Plausible Deniability (Duress Password & Decoy Database).
    - [ ] "Panic Button" (Key shredding).
    - [ ] Mesh Networking support (Bluetooth LE / Wi-Fi Direct).
    - [ ] Volunteer Credential verification logic.
- **Success Criteria:** Application functions without Internet (via Mesh), and data can be instantly wiped securely.

## 📱 Milestone 5: Mobility (v1.0.0)
**Focus:** Mobile Release & Polish.
Ready for public use.

- **Goals:**
    - [ ] Android & iOS builds.
    - [ ] Mobile-optimized UI/UX.
    - [ ] File sharing support.
    - [ ] Audit & Performance optimization.
- **Success Criteria:** Published APK/IPA that works seamlessly with Desktop clients.

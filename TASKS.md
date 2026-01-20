# Ratatoskr Implementation Plan

## Phase 1: Foundation (Core & SOS)
Goal: Establish the architectural skeleton, basic P2P connectivity, and the critical "Black Box SOS" feature.

- [x] **Project Setup**
    - [x] Initialize Monorepo (Core, Server, Desktop).
    - [x] Configure Rust Workspace.
    - [x] Initialize Tauri + Svelte frontend.
- [x] **Core Library (`ratatoskr-core`)**
    - [x] Define basic data models (`SosPayload`, `GeoLocation`).
    - [x] Implement ECIES Encryption (X25519 + AES-GCM) for anonymous SOS.
    - [x] Setup `libp2p` Swarm with GossipSub and mDNS.
- [x] **Relay Node (`ratatoskr-server`)**
    - [x] Implement basic server that listens to P2P network.
    - [x] Subscribe to `ratatoskr-sos` topic and log incoming packets.
- [x] **Desktop Client (`ratatoskr-desktop`)**
    - [x] Integrate Rust Core with Tauri.
    - [x] Implement background P2P thread.
    - [x] Create "Tactical UI" with Svelte & Tailwind/CSS.
    - [x] Connect UI "SOS Button" to Core networking logic.

## Phase 2: Decentralized Identity (DID)
Goal: Allow users to exist independently of any server.

- [x] **Identity Generation**
    - [x] Implement Ed25519 Keypair generation/storage (KeyVault).
    - [x] Create `DidDocument` structure (W3C standard compatible - baseline hex ID implemented).
- [x] **Registration flow**
    - [x] Generate a unique "Ratatoskr ID" (e.g., `did:rat:zABC...`).
    - [x] Implement "First Run" wizard in UI (Create Account / Import Seed).
    - [x] Implement Mnemonic recovery (BIP-39).
    - [x] Add secure backup export (.txt).
- [x] **Routing**
    - [x] Implement DHT (Kademlia) for finding peers by ID.
    - [ ] Allow publishing "Mailbox" addresses to the DHT/Network.
- [ ] **Privacy & Anonymity**
    - [ ] Implement Disposable Inboxes (Ephemeral burner DIDs).
    - [ ] Implement Circuit Breaker for public gateways (adaptive PoW).
    - [ ] Implement Blocklist / Gatekeeper settings.
- [ ] **Digital Legacy**
    - [ ] Implement Shamir's Secret Sharing for key sharding.
    - [ ] Create "Guardian" invitation and shard distribution flow.

## Phase 3: Secure Chat & Efficient Communication
Goal: Enable standard E2EE messaging with "Inbox Zero" architecture and multi-device support.

- [ ] **Messaging Protocol**
    - [ ] Implement Double Ratchet Algorithm (Signal Protocol style).
    - [x] Define `ChatMessage` structure with Semantic Types (`Direct`, `Ephemeral`, `Transactional`).
    - [x] Implement `TTL` logic for ephemeral messages (backend cleanup + UI removal).
    - [x] Implement bidirectional chat history (sender and recipient visibility).
    - [x] Implement Reply/Quote system.
- [x] **Inbox Zero Logic**
    - [x] Implement `GarbageCollector` background service.
    - [x] Implement `ActionRequired` and `Done` state logic.
- [ ] **Data & Sync**
    - [ ] Implement Large File Transfer (IPFS-style chunking).
    - [ ] Implement Multi-Device Sync using CRDTs.
- [ ] **Mailbox / Offline Delivery**
    - [ ] Implement "Blind Storage" logic on Relay Server.
    - [ ] Implement Client logic to poll Mailbox for new messages.
- [x] **Local Storage**
    - [x] Setup SQLite database with SQLx.
    - [ ] Implement "Plausible Deniability" (Decoy password vs Real password).

## Phase 4: Resilience & Governance
Goal: Operate without standard internet and manage community trust.

- [ ] **Mesh Networking**
    - [ ] Research & Implement Bluetooth Low Energy (BLE) transport for `libp2p`.
    - [ ] Enable peer discovery via local Wi-Fi multicast.
- [ ] **The Plague Protocol**
    - [ ] Implement trust-graph based reputation scoring.
    - [ ] Create "Quarantine" logic for infected nodes (silent muting).
- [ ] **Maintenance**
    - [ ] Implement P2P Update System (viral patching).
- [ ] **Trust Network**
    - [ ] Implement `VolunteerCredential` issuing and verification.

## Phase 5: Media, Mobile & Polish
Goal: High-performance real-time communication and mobile release.

- [ ] **Real-time Media**
    - [ ] Implement 1-on-1 A/V calls (WebRTC/libp2p stream).
    - [ ] Implement Group A/V calls via Blind SFU (Relay nodes).
- [ ] **Mobile Port**
    - [ ] Adapt UI for Mobile (Responsive).
    - [ ] Configure Tauri for Android/iOS build.
- [ ] **Audit & Launch**
    - [ ] Security audit of crypto and P2P implementation.
    - [ ] Public Beta release.

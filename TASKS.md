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

## Phase 3: Secure Chat & Storage
Goal: Enable standard E2EE messaging between users (User-to-User).

- [ ] **Messaging Protocol**
    - [ ] Implement Double Ratchet Algorithm (Signal Protocol style) for perfect forward secrecy.
    - [ ] Define `ChatMessage` payload structure.
- [ ] **Mailbox / Offline Delivery**
    - [ ] Implement "Blind Storage" logic on Relay Server (store encrypted blobs for ID).
    - [ ] Implement Client logic to poll Mailbox for new messages.
- [ ] **Local Storage**
    - [ ] Setup SQLite database with SQLCipher encryption.
    - [ ] Implement "Plausible Deniability" (Decoy password vs Real password).

## Phase 4: Resilience & Mesh
Goal: Operate without standard internet.

- [ ] **Mesh Networking**
    - [ ] Research & Implement Bluetooth Low Energy (BLE) transport for `libp2p`.
    - [ ] Enable peer discovery via local Wi-Fi multicast (already partially done via mDNS).
- [ ] **Trust Network**
    - [ ] Implement `VolunteerCredential` issuing and verification.
    - [ ] Create UI for Organizations to manage volunteers.

## Phase 5: Mobile & Polish
Goal: User-friendly mobile app.

- [ ] **Mobile Port**
    - [ ] Adapt UI for Mobile (Responsive).
    - [ ] Configure Tauri for Android/iOS build.
    - [ ] Test background services on mobile.
- [ ] **File Sharing**
    - [ ] Implement chunked file transfer via P2P streams.

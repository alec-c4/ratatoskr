# Ratatoskr Implementation Roadmap

## ✅ Phase 1: Foundation (Core & SOS)
**Status:** Complete
**Goal:** Establish architectural skeleton, P2P connectivity, and "Black Box SOS".

- [x] **Project Setup** (Monorepo, Workspace, Tauri+Svelte).
- [x] **Core Library** (Models, ECIES Encryption, libp2p Swarm).
- [x] **Relay Node** (Basic listener, GossipSub).
- [x] **Desktop Client** (Tactical UI, Background P2P thread, SOS logic).

## 🚧 Phase 2: Identity & Routing
**Status:** In Progress
**Goal:** Self-Sovereign Identity and decentralized discovery.

- [x] **Identity Management**
    - [x] Ed25519 Keypair generation/storage (`KeyVault`).
    - [x] BIP-39 Mnemonic recovery.
    - [x] Secure Backup Export (.txt).
- [x] **Routing (DHT)**
    - [x] Implement Kademlia DHT (`libp2p-kad`).
    - [x] Discovery via mDNS and Bootstrap nodes.
- [ ] **Privacy & Anonymity**
    - [ ] **Disposable Inboxes (Burner IDs):**
        - [ ] Implement HD Wallet derivation (BIP-32 style) for generating sub-keys from master seed.
        - [ ] Add `burn_after` field to DID Document.
        - [ ] UI: "Generate Temporary Contact Link" flow.
    - [ ] **Gatekeeper System:**
        - [ ] Implement `Blocklist` storage in SQLite.
        - [ ] Logic to drop messages from blocked DIDs at the network level.
        - [ ] "Allow Anonymous" toggle in settings (reject unsigned/unknown envelopes).
    - [ ] **Circuit Breaker:**
        - [ ] Implement Hashcash (Proof-of-Work) validation for incoming anonymous messages.
        - [ ] Dynamic difficulty adjustment based on inbox flood rate.

## 🔮 Phase 3: Secure Chat & Efficiency
**Status:** Active
**Goal:** E2EE messaging, "Inbox Zero" workflow, and multi-device sync.

- [x] **Core Messaging**
    - [x] `ChatMessage` structure with Semantic Types.
    - [x] SQLite Storage & History.
    - [x] UI: Two-pane chat, Replies, Self-messages.
    - [x] Inbox Zero: TTL auto-deletion (backend + UI).
- [ ] **Advanced Cryptography (Double Ratchet)**
    - [x] Implement **X3DH** (Extended Triple Diffie-Hellman) for initial key exchange.
    - [x] Implement **Double Ratchet** session management (root key, chain keys).
    - [x] Store session states securely in SQLite (using `sqlcipher` or application-level encryption).
    - [x] Header Encryption (hide routing metadata).
    - [x] **Bundle Exchange:** DHT-based PreKeyBundle publishing and retrieval.
    - [x] **Direct Messaging:** GossipSub topic `ratatoskr-direct` for real-time E2EE chat.
- [ ] **Inbox Zero Logic**
    - [ ] **Semantic Actions:**
        - [ ] Implement `ActionRequired` pinning logic.
        - [ ] UI for "Defer" (Snooze) -> hide message until timestamp.
        - [ ] UI for "Delegate" -> forward message and track status.
    - [ ] **Integrations:**
        - [ ] Define JSON Schema for `Transactional` messages (e.g., Bank Alert, Server Log).
        - [ ] Create Webhook Gateway for external services to push messages to local node.
- [ ] **Digital Legacy (Guardians)**
    - [ ] **Sharding:** Implement Shamir's Secret Sharing (Split seed into N parts).
    - [ ] **Guardian Protocol:**
        - [ ] "Invite Guardian" flow (sends encrypted shard).
        - [ ] "Accept Guardian" flow (stores shard securely).
    - [ ] **Recovery Switch:**
        - [ ] Logic for Guardians to publish "Shard Reveal".
        - [ ] Client logic to reconstruct Master Key from K revealed shards.
- [ ] **Offline Delivery (Mailbox)**
    - [ ] **Server:** Implement "Blind Storage" (store encrypted blobs keyed by Recipient DID).
    - [ ] **Protocol:** `RequestMessageStore` and `FetchMessages` commands in `libp2p`.
    - [ ] **Client:** Polling logic to check Mailbox when online.

## 🛡️ Phase 4: Resilience & Governance
**Goal:** Operate without standard internet and manage community trust.

- [ ] **Data & Sync (CRDTs)**
    - [ ] **Device Cluster:** Link mobile and desktop via QR code (share Master Key).
    - [ ] **State Sync:** Implement CRDTs (using `automerge` or `crdt`) to sync message history and read states between devices without a central server.
    - [ ] **Large Files:** Implement chunked file transfer stream (IPFS-style DAG) for media >10MB.
- [ ] **Mesh Networking**
    - [ ] **Bluetooth LE:** Integrate `libp2p-bluetooth` or platform-specific bindings for local discovery.
    - [ ] **Wi-Fi Direct:** Android/Linux specific implementation for off-grid connection.
- [ ] **The Plague Protocol (Reputation)**
    - [ ] **Trust Graph:** Store "Trust Scores" for contacts in DB.
    - [ ] **Reporting:** "Report Spam" button signs a `MalfeasanceProof` against the sender.
    - [ ] **Quarantine:** Logic to drop gossip propagation from nodes with Score < Threshold.
    - [ ] **Jury:** Random selection protocol for resolving disputes (advanced).
- [ ] **Unstoppable Updates**
    - [ ] Implement P2P file distribution for binary updates.
    - [ ] Verify updates against Foundation's offline GPG key.

## 📱 Phase 5: Mobile & Polish
**Goal:** Public release and mobile parity.

- [ ] **Mobile Port (Tauri Mobile)**
    - [ ] Configure Android manifest & permissions (Foreground Service for P2P).
    - [ ] Configure iOS capabilities (Network Extensions).
    - [ ] Adapt UI: Bottom navigation, touch gestures (swipe to reply).
- [ ] **Real-time Media**
    - [ ] **1-on-1:** WebRTC signaling over P2P connection + direct media stream.
    - [ ] **Group Calls:** Implement "Blind SFU" logic on Relay nodes (forwarding encrypted SRTP packets).
- [ ] **Security Audit**
    - [ ] Fuzz testing of P2P handlers.
    - [ ] Formal review of Double Ratchet implementation.
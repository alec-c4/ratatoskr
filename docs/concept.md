# Ratatoskr: Resilient Decentralized Messenger

## Vision
Ratatoskr is an open-source, decentralized, and censorship-resistant messenger designed for high-stakes environments, including natural disasters, civil emergencies, and areas with restricted internet access. 

Inspired by the messenger of the world tree in Norse mythology, Ratatoskr ensures that information flows even when the "trunk" of the internet is severed.

## Core Principles

1. **Unblockable Architecture:** No central servers. Communication happens over a Peer-to-Peer (P2P) network using `libp2p` with pluggable transports (TCP, QUIC, WebSocket) to bypass DPI.
2. **Self-Sovereign Identity (SSI):** Users own their IDs. Identity and routing metadata are managed via blockchain (DIDs), allowing users to migrate between relay nodes without losing their history or contacts.
3. **Resilience & Mesh Networking:** Works in total internet blackout scenarios by utilizing Mesh networks (Bluetooth, Wi-Fi Direct, LoRa).
4. **Black Box SOS:** A specialized emergency protocol for sending encrypted SOS signals that can only be decrypted by verified humanitarian organizations, protecting victims from potential aggressors.
5. **Privacy by Default:** End-to-end encryption (E2EE) using modern cryptographic standards (Double Ratchet, Ed25519).

## Technology Stack

- **Core Logic:** Rust (Performance, Memory Safety, P2P efficiency).
- **Frontend:** Svelte + Tailwind CSS (Lightweight and fast).
- **Desktop/Mobile Wrapper:** Tauri v2 (Low resource footprint).
- **Networking:** `libp2p` (DHT for discovery, GossipSub for channels).
- **Identity:** Decentralized Identifiers (DIDs) on an L2 Blockchain.
- **Local Storage:** SQLite (Encrypted).

## Civil Defense Features: "Black Box SOS"
In emergency scenarios (war, natural disasters), Ratatoskr provides a "Silent SOS" mode:
- **Asymmetric Encryption:** SOS messages are encrypted with the public keys of trusted aid organizations.
- **Metadata Anonymity:** Routing hides the source of the SOS signal to prevent triangulation by hostile actors.
- **Burst Transmission:** Short, encrypted radio bursts to minimize detection.

## Plausible Deniability & Data Safety
To protect users under duress (e.g., capture, torture, or border checks), Ratatoskr implements **"Duress Mode"**:
- **Decoy Profiles:** Users can maintain two passwords. One unlocks the real "Shadow" profile, while the other unlocks a "Decoy" profile populated with innocuous, non-sensitive chats.
- **Panic Wipe:** Entering a specific "Panic PIN" or triggering a hardware sequence (e.g., rapid power button presses) instantly shreds the local encryption keys from memory and disk, rendering the database permanently unrecoverable.
- **Ephemeral Messaging:** Granular auto-delete timers for messages (burn-on-read or time-based) ensuring history is not kept longer than necessary.

## Anti-Spam & Governance: "The Plague Protocol"

To manage malicious actors without central authority, Ratatoskr implements a graph-based reputation system:

1.  **Infection & Quarantine:**
    - If a user broadcasts malware or spam, peers can flag them as "Infected".
    - This flag propagates through the Web of Trust. If your trusted contacts flag someone, your node automatically downgrades their reputation.
    - **"Leper Colony":** Nodes with low reputation are isolated. Their messages are dropped by relay nodes, effectively muting them in public channels.
    - **Contagion:** Interacting with an "Infected" node lowers your own reputation score, discouraging support for bad actors.

2.  **SOS Immunity:**
    - **Fundamental Right:** Even "Infected" nodes retain the ability to broadcast `SOS` signals. Saving a life takes precedence over social moderation.

3.  **Censorship Resistance:**
    - **Local View:** Reputation is subjective. A political group cannot globally ban an opponent; they can only ban them *from their own cluster*.
    - **Anonymous Jury:** For global disputes, a mechanism selects random, disinterested nodes to act as an anonymous jury, preventing targeted persecution.

## Inbox Zero Protocol (Architecture of Attention)

Ratatoskr replaces the outdated "chronological list" model with an intent-based architecture designed to eliminate clutter:

1.  **Semantic Message Types:**
    - **Direct (Human):** Standard messages requiring attention.
    - **Ephemeral (OTP/Secret):** Messages with a strict **TTL (Time-To-Live)**. They self-destruct after reading or a set time (e.g., 2FA codes), never clogging the database.
    - **Transactional:** Information that is useful *once* (receipts, alerts). Auto-archived immediately after reading.
    - **Feed:** Low-priority updates. Stored in a ring buffer (oldest replaced by newest), never triggering push notifications.

2.  **Stateful Threads (GTD Workflow):**
    - Messages are treated as tasks.
    - **States:**
        - `Open`: Requires action.
        - `Done`: Completed and archived.
        - **`Defer` (Snooze):** Hide until a specific time or location.
        - **`Decline`:** Reject the request (sender is notified).
        - **`Delegate`:** Forward the action item to another contact with tracking.
    - **Priority:** High-priority messages override 'Do Not Disturb' or 'Defer' rules based on relationship context.

3.  **The "Postage Stamp" (Anti-Spam):**
    - To contact a user for the first time, a sender must attach a cryptographic **Proof-of-Work (PoW)** or a small token stake.
    - This creates a computational cost for spam, making mass mailing economically unviable while remaining free for normal humans.

4.  **Extensible Protocols:**
    - The message type system is flexible. Developers can define custom protocols (e.g., "Voting", "Payment", "Live Location") with their own retention policies and action buttons.
    - **Open Integrations:** External services (banks, monitoring systems) can send encrypted notifications directly to a user's DID, automatically categorized as `Transactional` to keep the Inbox clean.

5.  **Seamless Experience:**
    - **Multi-Device:** Sync your chats across phone and laptop without relying on a cloud server.
    - **Calls:** Crystal clear Voice and Video calls, end-to-end encrypted. Group calls are supported via volunteer relays.
    - **Unstoppable Updates:** Even if our website is blocked, the app updates itself through the mesh network.

## Digital Legacy & Guardians

Recognizing that users are mortal, Ratatoskr includes a **Social Recovery** mechanism:
- **Shamir's Secret Sharing:** A user can split their master key into fragments and distribute them to trusted contacts ("Guardians").
    - **Dead Man's Switch:** If Guardians initiate recovery, the user receives a high-priority alert. If the user does not cancel the process within a set timeframe (proving they are incapacitated or deceased), the Guardians can reconstruct the key to recover the account for the family.

## The Spectrum of Identity

Ratatoskr recognizes that different conversations require different levels of trust:

1.  **Anonymous (Ghost Mode):**
    - **Burner IDs:** Generate a temporary identity for a single conversation.
    - **Disposable Inboxes:** Create a contact link that expires after a set time (e.g., 24h) or message count. Senders are explicitly notified: *"This is a temporary address. It may become unreachable at any time."*
    - **Public Gateways (Circuit Breaker):** For website contact forms. If a public address is flooded with spam, it automatically triggers a "lockdown" (drastically increasing Proof-of-Work difficulty) or rotates the underlying ID without affecting the user's main profile.
    
2.  **Pseudonymous (Web of Trust):**
    - Standard usage. Identify by persistent Public Key/Nickname. Trust is earned through reputation and mutual contacts.

3.  **Verified (KYC/Bright Mode):**
    - **Verifiable Credentials (VC):** Users can attach cryptographically signed attestations (e.g., "Verified Journalist", "Bank Employee") from trusted Issuers.
    - **Use Case:** High-value transactions, official press comms, legal agreements.

## Privacy & Access Control

- **Gatekeeper Settings:** Users have granular control over who can contact them.
    - **Allow Anonymous:** (On/Off). If Off, only messages from contacts or verified identities are accepted.
    - **Blocklist:** Silent drop of messages from specific DIDs.
    - **Whitelisting:** "Panic Mode" where only pre-approved contacts can reach the user.

## Implementation Roadmap
### Phase 1: Foundation (Current)
- Establish Rust Core and P2P connectivity.
- Local-only mDNS peer discovery.
- Basic E2EE messaging.

### Phase 2: Decentralized Identity
- Blockchain integration for DID registration.
- DHT-based routing.

### Phase 3: Resilience
- Mesh networking implementation (Bluetooth/BLE).
- Black Box SOS protocol.

### Phase 4: Scaling & UI
- Mobile applications (Android/iOS).
- Advanced group management and file sharing.

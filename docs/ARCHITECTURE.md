# Ratatoskr Architecture Reference

## 1. System Overview

Ratatoskr operates as a decentralized network with a hybrid architecture combining Peer-to-Peer (P2P) direct messaging, Distributed Ledger Technology (DLT) for identity, and a "Store-and-Forward" network for offline delivery.

### High-Level Components

```mermaid
graph TD
    User[User Client] <-->|P2P / Gossip| Relay[Relay Node]
    User <-->|Offline Msg| Mailbox[Mailbox Node]
    User <-->|Identity| Blockchain[L2 Identity Registry]
    Relay <-->|Sync| Relay
    Org[Organization Node] -->|Validate| Volunteer[Volunteer Client]
```

## 2. Node Roles

A single physical server or device can perform multiple roles, but logically they are distinct:

### A. Client Node (User/Volunteer)
- **Platform:** Mobile (Android/iOS), Desktop (Tauri), Embedded.
- **Function:** 
  - Key management (Ed25519/X25519).
  - E2EE Encryption/Decryption.
  - Local Storage (SQLite) of message history.
  - **Does NOT** listen on public ports (typically behind NAT).

### B. Relay Node (The "Pipe")
- **Function:** High-bandwidth packet routing.
- **Protocol:** GossipSub (libp2p).
- **Storage:** Ephemeral (RAM only). Holds messages for seconds to propagate them to subscribers.
- **Use Case:** Public chat channels, "Black Box SOS" broadcasting.

### C. Mailbox Node (The "Storage")
- **Function:** Stores encrypted messages for offline users.
- **Protocol:** Request/Response (Direct).
- **Storage:** Persistent (Disk).
- **Privacy:** **Blind Storage**. The Mailbox knows *who* the message is for (Public Key), but not *who* sent it or *what* is inside. 
- **Anti-Spam:** Requires a Proof-of-Work token or a whitelisted signature to accept a message for storage.

### D. Organization/Dispatch Node
- **Function:** Command center for emergency response.
- **Capabilities:**
  - Holds high-privilege Private Keys to decrypt SOS signals.
  - Issues **Verifiable Credentials (VCs)** to Volunteers (e.g., "Certified Medic").
  - Manages the "Web of Trust".

---

## 3. Data Flow Scenarios

### Scenario 1: Offline Messaging (Async)
1. **Alice** wants to send a message to **Bob**.
2. Alice resolves Bob's DID (Decentralized Identifier) from the Blockchain/DHT.
3. Bob's DID Document contains a list of **Service Endpoints** (his chosen Mailboxes).
   - `did:ratatoskr:bob123 -> { mailbox: "node-5.ratatoskr.net" }`
4. Alice connects to `node-5` and uploads an encrypted blob addressed to Bob.
5. `node-5` stores the blob.
6. **Bob** comes online, connects to `node-5`, authenticates with his Private Key, and downloads pending blobs.
7. Bob deletes the blobs from the server after download.

### Scenario 2: Black Box SOS (Emergency)
1. **User** activates SOS Mode.
2. Client encrypts location/status with the **Organization's Public Key**.
3. Client wraps this payload in an **Anonymous Envelope** (ephemeral keys).
4. Client broadcasts the envelope via **GossipSub** to *any* available peer (Mesh or Internet).
5. Peers relay the packet blindly until it reaches an **Organization Node**.
6. Organization decrypts the packet and dispatches help.

---

## 4. Trust & Access Control (Volunteers & Reputation)

We use a **Web of Trust (WoT)** model anchored by Organizations and a decentralized reputation system ("Plague Protocol").

### A. Volunteer Verification
- **Verifiable Credentials (VC):**
  - An Organization signs a certificate: `{ "subject": "Volunteer_PublicKey", "role": "Medic", "exp": 2026-01-01 }`.
  - This certificate is stored on the Volunteer's device.
- **Dispatching:**
  - When an Organization receives an SOS, they look for active peers with valid VCs.
  - The Organization sends the coordinates *only* to the selected Volunteer, encrypted with the Volunteer's key.

### B. The Plague Protocol (Reputation System)
To manage spam and malicious actors without central authority:
- **Infection:** Nodes can flag peers as "Infected" (malicious). This flag propagates through the Web of Trust.
- **Quarantine (Leper Colony):** Messages from nodes with a low reputation score are dropped by relays for public channels.
- **SOS Immunity:** `SOS` signals are **exempt** from reputation filtering. Saving a life takes precedence over social moderation.

---

## 7. Storage Layer (User Device)

- **Engine:** SQLite (via SQLx).
- **Encryption:** SQLCipher or Application-level AES-GCM encryption of database rows.
- **Schema:**
  - `contacts`: DIDs, Public Keys, Nicknames, Reputation Score.
  - `messages`: 
    - `id`: UUID.
    - `content`: Encrypted blob.
    - `type`: `Direct` | `Ephemeral` | `Transactional` | `Feed`.
    - `ttl`: Timestamp for auto-deletion.
    - `status`: `Unread` | `ActionRequired` | `Done`.
  - `credentials`: Own certificates and certificates of trusted organizations.

### A. Extensible Message Format
Messages are not just text. They are structured objects defined by a Schema ID.
```json
{
  "header": {
    "type": "urn:ratatoskr:protocol:ephemeral",
    "ttl": 3600,
    "priority": "high"
  },
  "body": "Encrypted Payload..."
}
```

### B. Digital Legacy (Key Sharding)
- **Algorithm:** Shamir's Secret Sharing (SSS) over Ed25519 seed.
- **Protocol:**
  1. User selects N Guardians. Threshold K is set (e.g., 3 of 5).
  2. Shards are encrypted with Guardians' Public Keys and stored in their Mailboxes.
  3. **Recovery:** Guardians publish "Shard Reveal" transactions. Once K reveals are seen, the Client (on a relative's device) can reconstruct the seed.

---

## 9. Advanced Capabilities

### A. Multi-Device Sync (CRDT)
To support a seamless experience across mobile and desktop:
- **Device Clusters:** Multiple devices share the same Identity (Master Key derives Sub-keys).
- **State Sync:** Message history and read statuses are synchronized using **CRDTs (Conflict-free Replicated Data Types)**. This allows offline modifications on multiple devices to merge automatically without conflicts when connectivity is restored.

### B. Real-Time Media (A/V Calls)
- **1-on-1 Calls:** Direct P2P streams via `libp2p` (using QUIC transport).
- **Group Calls:** To avoid bandwidth saturation in a full mesh, volunteer Relay Nodes act as **Blind SFUs (Selective Forwarding Units)**. They forward encrypted media packets between participants without having the keys to decrypt the audio/video streams.

### C. Large File Transfer
- **Protocol:** IPFS-style chunking.
- **Delivery:** Files are not sent through the chat channel. The sender uploads encrypted chunks to a swarm of Mailbox nodes (or directly to the receiver via a data stream). The chat message contains only the **Content Hash (CID)** and decryption key.

### D. Censorship-Resistant Updates
- **Viral Patching:** The application can download signed updates from other peers in the network, bypassing blocked App Stores or websites.
- **Verification:** Updates are signed by the Foundation's offline root key.

The client implements logic to automatically manage message lifecycles:
1.  **Garbage Collection:** A background task runs periodically to delete expired `Ephemeral` messages.
2.  **Auto-Archive:** `Transactional` messages move to `Archive` state immediately upon `Read` event.
3.  **Focus View:** The UI prioritizes threads with `status = ActionRequired` or `Unread` from Human contacts.


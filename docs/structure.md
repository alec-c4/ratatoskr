# Ratatoskr Codebase Decomposition

This document maps the logical architecture to the physical code structure of the Monorepo. Use this as a guide to locate features and understand dependency flows.

## 📂 Repository Root

| File/Dir | Purpose |
| :--- | :--- |
| `Cargo.toml` | Rust Workspace definition. Manages shared dependencies across Core, Server, and Desktop. |
| `package.json` | Node.js dependencies for tooling (Lefthook, Prettier) and Frontend. |
| `lefthook.yml` | Git hooks configuration (Pre-commit checks). |
| `mise.toml` | Task runner and tool version manager configuration. |
| `.github/` | CI/CD workflows (GitHub Actions). |
| `scripts/` | Utility scripts (e.g., `run_testnet.sh` for multi-node simulation). |

---

## 🧠 Core Library (`ratatoskr-core`)
**Role:** The shared brain. Contains all protocol logic, cryptography, and networking. Used by both Client and Relay Server.

### `src/`
- **`lib.rs`**: Module exports and library initialization.
- **`models.rs`**: Data structures exchanged over the network and stored in DB.
    - `ChatMessage`: The central object (includes `msg_type`, `ttl`, `reply_to`).
    - `SosPayload`: Emergency signal structure.
    - `EncryptedMessage`: Protocol buffer for X3DH and Double Ratchet payloads.
- **`network.rs`**: The P2P Networking Stack (`libp2p`).
    - `RatatoskrBehavior`: Combines `GossipSub` (Chat/SOS) and `Kademlia` (DHT Routing).
    - `NetworkCommand/Event`: Channels for async communication. Supports Direct Messages and Bundle Exchange.
- **`crypto.rs`**: Cryptographic primitives (ECIES, legacy).
- **`ratchet.rs`**: Double Ratchet implementation (Diffie-Hellman + Hash Ratchets).
- **`x3dh.rs`**: Extended Triple Diffie-Hellman key exchange logic.
- **`messaging.rs`**: High-level service orchestrating encryption, session management, and storage.
- **`key_vault.rs`**: Identity Management.
    - Ed25519 Key generation.
    - BIP-39 Mnemonic recovery logic.
    - Secure file I/O.
- **`storage.rs`**: Persistence Layer.
    - SQLite connection pool (`sqlx`).
    - `GarbageCollector`: Logic for deleting expired TTL messages.
    - CRUD operations for Contacts, Messages, Sessions, and PreKey Bundles.
- **`access_control.rs`**: Authorization.
    - `VolunteerCredential` definitions.
    - *Future:* Reputation system logic ("Plague Protocol").

### `migrations/`
- SQL files for database schema versioning (e.g., `20240101...init.sql`).

---

## 🖥️ Desktop Client (`ratatoskr-desktop`)
**Role:** The user-facing application. Wraps Core with a GUI.

### `src-tauri/` (Rust Backend)
- **`src/lib.rs`**: The Bridge.
    - **Tauri Commands:** Functions callable from JS (`send_message`, `panic_wipe`).
    - **State Management:** Holds `AppState` (Network channels, DB connection).
    - **Background Threads:** Spawns the P2P node and the Event Listener loop.
- **`tauri.conf.json`**: App configuration (permissions, bundle settings).

### `src/` (Svelte Frontend)
- **`routes/+page.svelte`**: The Main UI (currently monolithic, planned for split).
    - **Sidebar:** Navigation (Chats, Contacts, Settings).
    - **Chat View:** Message rendering, bubbles, input area.
    - **Settings:** Identity management, Backup, Panic Button.
- **`lib/`**: (Planned) Shared Svelte components (`MessageBubble.svelte`, `ContactItem.svelte`).

---

## 📡 Relay Server (`ratatoskr-server`)
**Role:** Infrastructure node. Always-on peer for bootstrapping and routing.

### `src/`
- **`main.rs`**: Application Entry point.
    - Initializes `ratatoskr-core`.
    - Configures P2P in **Server Mode** (DHT Provider).
    - Logs traffic (SOS signals).
    - *Future:* Blind Mailbox storage implementation.

---

## 🔗 Logical Flow Examples

### 1. Sending a Message
1.  **UI (`+page.svelte`):** User types text -> calls `invoke("send_message")`.
2.  **Tauri (`lib.rs`):** 
    - Loads Identity from `KeyVault`.
    - Saves message to SQLite via `Storage`.
    - Sends `NetworkCommand::Broadcast` to the Network Channel.
3.  **Core (`network.rs`):** 
    - Picks up command.
    - Publishes to `libp2p` GossipSub topic.

### 2. Receiving a Message
1.  **Core (`network.rs`):** `libp2p` receives packet -> Sends `NetworkEvent::MessageReceived`.
2.  **Tauri (`lib.rs`):** 
    - Listener loop catches event.
    - Saves to SQLite (Status: Unread).
    - Emits Tauri Event `msg-received`.
3.  **UI (`+page.svelte`):** 
    - `listen()` triggers.
    - Updates `chatMessages` array.
    - DOM updates to show new bubble.

### 3. Panic Wipe
1.  **UI:** User clicks "Destroy". Calls `invoke("delete_identity")`.
2.  **Tauri:** Deletes `identity.key` from disk.
3.  **UI:** Reloads window (`window.location.reload()`), resetting all JS state.

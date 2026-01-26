# Ratatoskr Implementation Plan

## Phase 3: Secure Chat & Efficiency

### Stage 1: Advanced Cryptography (Double Ratchet & X3DH) - COMPLETED
**Goal:** Implement the Signal Protocol for end-to-end encryption.
- [x] **Double Ratchet:** Implemented `ratchet.rs` with Diffie-Hellman and Hash ratchets.
- [x] **X3DH:** Implemented `x3dh.rs` for initial key exchange.
- [x] **Session Storage:** Updated `storage.rs` to persist ratchet sessions.
- [x] **PreKey Storage:** Updated `storage.rs` to manage Signed and One-Time PreKeys.
- [x] **Messaging Service:** Created `messaging.rs` to orchestrate encryption/decryption.
- [x] **Integration Test:** Verified full flow in `tests/messaging_tests.rs`.

### Stage 2: Network Integration (Next) - COMPLETED
**Goal:** Connect the crypto layer to the P2P network.
- [x] **Protocol Update:** Update `NetworkCommand` and `NetworkEvent` to support `EncryptedMessage`.
- [x] **Bundle Exchange:** Implement DHT-based or Gossip-based PreKeyBundle distribution.
- [x] **Tauri Integration:** Connect `MessagingService` to the Desktop UI.

### Stage 3: Inbox Zero (Blind Mailbox)
**Goal:** Offline message delivery.
- [ ] **Mailbox Protocol:** Define `StoreMessage` and `FetchMessages` commands.
- [ ] **Server Implementation:** Build `ratatoskr-server` as a high-availability relay/mailbox.
- [ ] **Client Polling:** Implement mailbox polling in Desktop client.

## Phase 2: Identity & Routing (Refinement)
- [ ] **PreKey Generation:** Ensure clients generate and rotate PreKeys automatically.
- [ ] **Identity Backup:** Improve backup format to include PreKeys? (Maybe not, strictly identity).


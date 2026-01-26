# Ratatoskr

**Resilient Decentralized Messenger**

![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)
![Status: Prototype](https://img.shields.io/badge/Status-Prototype-orange.svg)

> **Mission:** To ensure information flows even when the "trunk" of the internet is severed.
>
> **Original Concept:** Alexey Poimtsev

## 📜 Documentation & Governance

- **[Concept & Vision](docs/concept.md):** Detailed overview of features (Black Box SOS, Plague Protocol).
- **[Architecture](docs/architecture.md):** Technical design (P2P, Encryption, DHT).
- **[Roadmap](docs/roadmap.md):** Granular implementation tasks and engineering goals.
- **[Milestones](docs/milestones.md):** Project release timeline.
- **[Project Structure](docs/structure.md):** Codebase decomposition and directory map.
- **[Ethics](docs/governance/ethics.md):** Our stance on surveillance and human rights.
- **[Foundation Charter](docs/governance/foundation.md):** Governance principles.
- **[Trademark Policy](docs/governance/trademark.md):** Rules for using the brand.
- **[SOS Governance](docs/governance/sos.md):** Verification protocol for aid organizations.

## 🛠️ Features

- **Unblockable:** P2P architecture using `libp2p` (GossipSub, Kademlia DHT).
- **Secure:** End-to-End Encryption (E2EE) by default.
- **Emergency Mode:** "Black Box SOS" protocol for anonymous distress signals.
- **Identity:** Self-Sovereign Identity (DID) with BIP-39 mnemonic recovery.
- **Resilience:** Designed for mesh networking and offline scenarios.

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js (v18+) & npm

### Development

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/alec-c4/ratatoskr.git
    cd ratatoskr
    ```

2.  **Run the Relay Server (Terminal 1):**

    ```bash
    cargo run -p ratatoskr-server
    ```

3.  **Run the Desktop Client (Terminal 2):**
    ```bash
    cd ratatoskr-desktop
    npm install
    npm run tauri dev
    ```

## 🤝 Contributing

We welcome contributions that align with our [Mission](docs/governance/foundation.md). 
Please read our [Architecture](docs/architecture.md) guide before submitting PRs.

## 📄 License

Code is licensed under the **GNU Affero General Public License v3.0** (AGPL-3.0).
See [LICENSE](LICENSE) for details.

The "Ratatoskr" name and logo are trademarks. See [Trademark Policy](docs/governance/trademark.md).
See [NOTICE](docs/notice.md) for attribution.

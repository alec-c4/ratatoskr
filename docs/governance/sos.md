# SOS Key Governance Protocol

The security of the "Black Box SOS" feature depends entirely on the trustworthiness of the public keys embedded in the application. This document outlines how organizations are verified and keys are managed.

## 1. Trust Model

- **SOS messages are encrypted** using the public keys of verified humanitarian organizations.
- **Goal:** Ensure only neutral, aid-focused entities can decrypt the location and identity of a distress signal.
- **Threat Model:** We assume adversaries may attempt to impersonate aid organizations to unmask victims.

## 2. Inclusion Process (The Whitelist)

To have a public key included in the Ratatoskr trusted bundle, an organization must pass the following checks:

### Step 1: Public Application
The organization must submit a public request (e.g., GitHub Issue or Foundation Form) including:
- Official website and physical headquarters address.
- Point of contact (PGP signed).
- Stated mission (must align with [Foundation Charter](foundation.md)).

### Step 2: Identity Verification
The Ratatoskr Foundation (or a delegated Trust Committee) verifies identity via:
- **Domain Verification:** DNS TXT records.
- **Cross-Signing:** Verification by another already-trusted entity (Web of Trust).
- **Out-of-Band Check:** Phone call or physical meeting with official representatives.

### Step 3: Community Review
- A "Request for Comments" (RFC) period of at least 14 days.
- The community investigates the organization's history for neutrality violations or data leaks.

## 3. Key Management

- **Rotation:** Organizations must rotate keys annually.
- **Audit Log:** All additions and removals of keys are recorded in the public git history and/or a transparency ledger.
- **Revocation:** 
  - Keys can be emergency-revoked if a private key compromise is suspected.
  - Revocation updates are propagated via the P2P network (high-priority gossip).

## 4. Key Removal

An organization will be removed if:
- It ceases operations.
- It is found to share decrypted SOS data with belligerent parties (military/police/intelligence).
- It fails to rotate compromised keys.

*Removal is permanent and publicly documented.*

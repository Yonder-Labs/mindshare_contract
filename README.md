# Agent Worker Smart Contract

A secure and verifiable smart contract system for managing worker agents in a Trusted Execution Environment (TEE) on NEAR Protocol. This contract provides verification, registration, and access control mechanisms for worker agents.

## Features

- **Worker Registration & Verification**
  - Secure registration of worker agents through TEE attestation
  - Verification of remote attestation quotes and collateral
  - Storage of worker checksums and codehashes

- **Access Control System**
  - Method-level access control based on verified worker codehashes
  - Owner-managed approval system for worker codehashes
  - Protected methods accessible only by verified agents

- **MPC Integration**
  - Integration with NEAR's Multi-Party Computation (MPC) for secure signing
  - Support for derived public keys
  - Cross-chain transaction signing capabilities

## Smart Contract Methods

### Core Methods

```rust
// Register a new worker with attestation data
pub fn register_worker(
    quote_hex: String,
    collateral: String, 
    checksum: String,
    tcb_info: String
) -> bool

// Sign trade with verified worker
#[payable]
pub fn sign_trade(quote: String) -> Promise

// Get worker information
pub fn get_worker(account_id: AccountId) -> Worker
```

### Access Control

```rust
// Approve a worker codehash (owner only)
pub fn approve_codehash(codehash: String)

// Check if caller has approved codehash
pub fn require_approved_codehash()
```

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test?

```bash
cargo test
```

## Deployment

```bash
cargo near deploy <account-id>
```

## Security Considerations

- All sensitive methods are protected by worker verification
- Worker registration requires valid TEE attestation
- Access control is managed through codehash verification
- Owner-only administrative functions

## Technical Architecture

1. **Worker Registration Flow**
   - TEE generates attestation quote
   - Contract verifies quote authenticity
   - Worker codehash and checksum are stored
   
2. **Method Access Control**
   - Methods check caller's registered codehash
   - Only approved codehashes can access protected functions
   - Owner manages approved codehash list

## Useful Links

- [NEAR Rust SDK Documentation](https://docs.near.org/)
- [Chain Abstraction Telegram Group](https://t.me/chain_abstraction)
- [Shade Agent Reference](https://fringe-brow-647.notion.site/Shade-Agents-19a09959836d8091bb8febb318cc09fd#19a09959836d80618a0bec4b7effd0bc)
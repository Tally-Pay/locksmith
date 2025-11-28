# Locksmith

A native Solana program for trustless SPL token locking until a specified Unix timestamp.

## Overview

Locksmith allows users to lock any SPL token until a future date. Once locked, tokens cannot be withdrawn until the unlock timestamp passes. This is useful for:

- **Token vesting** - Lock team/investor tokens with scheduled releases
- **Liquidity locks** - Prove LP tokens are locked for a duration
- **Commitment mechanisms** - Lock tokens as collateral or proof of commitment

## Features

- **Trustless** - No admin can access locked tokens; only the owner can unlock after the timestamp
- **Any SPL token** - Works with any SPL token mint
- **Multiple locks** - Create unlimited locks per wallet using unique lock IDs
- **Minimal fees** - 0.15 USDC per lock creation
- **Compact** - ~116KB deployed binary (native Rust, no Anchor)

## Program Details

| | |
|---|---|
| **Program ID** | `A5vz72a5ipKUJZxmGUjGtS7uhWfzr6jhDgV2q73YhD8A` |
| **Network** | Devnet |
| **Fee** | 0.15 USDC per lock |
| **Max Lock Duration** | 10 years |

## SDK

A TypeScript SDK is available for easy integration:

```bash
cd sdk && npm install
```

### Usage

```typescript
import {
  getInitializeLockInstruction,
  getUnlockInstruction,
  findLockAccountPda,
  findLockTokenPda,
  findFeeVaultPda,
  fetchLockAccount,
  LOCKSMITH_PROGRAM_ADDRESS,
  FEE_USDC,
  USDC_MINT,
} from "./sdk/src";

// Derive PDAs
const [lockAccount] = await findLockAccountPda(owner, mint, lockId);
const [lockToken] = await findLockTokenPda(lockAccount);
const [feeVault] = await findFeeVaultPda();

// Create a lock
const lockIx = getInitializeLockInstruction({
  owner: ownerSigner,
  ownerTokenAccount,
  ownerUsdcAccount,
  mint,
  lockAccount,
  lockTokenAccount: lockToken,
  feeVault,
  amount: 1_000_000n,
  unlockTimestamp: BigInt(Math.floor(Date.now() / 1000) + 86400), // 24 hours
  lockId: 1n,
});

// Unlock after timestamp passes
const unlockIx = getUnlockInstruction({
  owner: ownerSigner,
  ownerTokenAccount,
  lockAccount,
  lockTokenAccount: lockToken,
  lockId: 1n,
});

// Fetch lock details
const lock = await fetchLockAccount(rpc, lockAccount);
console.log(`Locked: ${lock.data.amount} tokens until ${lock.data.unlockTimestamp}`);
```

## Instructions

| Instruction | Description |
|-------------|-------------|
| `InitializeConfig` | One-time setup of program config and USDC fee vault (admin only) |
| `TransferAdmin` | Transfer admin role to a new wallet |
| `WithdrawFees` | Admin withdraws accumulated USDC fees |
| `InitializeLock` | Create a new token lock with amount, unlock timestamp, and lock ID |
| `Unlock` | Release tokens after the unlock timestamp has passed |

## Building

```bash
# Check for errors (fast)
cargo check --manifest-path programs/locksmith/Cargo.toml

# Build deployable binary
cargo build-sbf --manifest-path programs/locksmith/Cargo.toml

# Run tests
cargo nextest run --manifest-path programs/locksmith/Cargo.toml
```

## Regenerating the SDK

If you modify the program:

```bash
# Regenerate IDL and SDK
npm run generate

# Or separately:
npm run generate:idl   # Shank IDL extraction
npm run generate:sdk   # Codama TypeScript generation
```

## Architecture

```
programs/locksmith/src/
├── lib.rs          # Entrypoint
├── instruction.rs  # Instruction enum with Shank macros
├── processor.rs    # Instruction handlers
├── state.rs        # Account structures (ConfigAccount, LockAccount)
└── error.rs        # Custom error codes

sdk/src/
├── index.ts        # SDK entry point
├── pdas.ts         # PDA derivation helpers
├── constants.ts    # USDC_MINT, FEE_USDC, etc.
└── generated/      # Codama-generated code
```

## PDA Seeds

| PDA | Seeds |
|-----|-------|
| Config | `["config"]` |
| Fee Vault | `["fee_vault"]` |
| Lock Account | `["lock", owner, mint, lock_id (u64 LE bytes)]` |
| Lock Token | `["lock_token", lock_account]` |

## Security

- Lock tokens are held in program-controlled escrow accounts
- Only the lock owner can unlock, and only after the timestamp
- USDC mint is hardcoded to prevent fake fee payments
- Fees are hardcoded and cannot be changed without program upgrade

## License

[MIT](LICENSE)

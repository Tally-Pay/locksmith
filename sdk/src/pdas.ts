import {
  getAddressEncoder,
  getProgramDerivedAddress,
  getU64Encoder,
  type Address,
  type ProgramDerivedAddress,
} from "@solana/kit";
import { LOCKSMITH_PROGRAM_ADDRESS } from "./generated";

// PDA seed constants (matching Rust)
const CONFIG_SEED = new TextEncoder().encode("config");
const FEE_VAULT_SEED = new TextEncoder().encode("fee_vault");
const LOCK_SEED = new TextEncoder().encode("lock");
const LOCK_TOKEN_SEED = new TextEncoder().encode("lock_token");

/**
 * Find the Config PDA
 * Seeds: ["config"]
 */
export async function findConfigPda(
  programAddress: Address = LOCKSMITH_PROGRAM_ADDRESS
): Promise<ProgramDerivedAddress> {
  return getProgramDerivedAddress({
    programAddress,
    seeds: [CONFIG_SEED],
  });
}

/**
 * Find the Fee Vault PDA (USDC token account for collecting fees)
 * Seeds: ["fee_vault"]
 */
export async function findFeeVaultPda(
  programAddress: Address = LOCKSMITH_PROGRAM_ADDRESS
): Promise<ProgramDerivedAddress> {
  return getProgramDerivedAddress({
    programAddress,
    seeds: [FEE_VAULT_SEED],
  });
}

/**
 * Find a Lock Account PDA
 * Seeds: ["lock", owner, mint, lock_id.to_le_bytes()]
 */
export async function findLockAccountPda(
  owner: Address,
  mint: Address,
  lockId: bigint | number,
  programAddress: Address = LOCKSMITH_PROGRAM_ADDRESS
): Promise<ProgramDerivedAddress> {
  const addressEncoder = getAddressEncoder();
  const u64Encoder = getU64Encoder();

  return getProgramDerivedAddress({
    programAddress,
    seeds: [
      LOCK_SEED,
      addressEncoder.encode(owner),
      addressEncoder.encode(mint),
      u64Encoder.encode(BigInt(lockId)),
    ],
  });
}

/**
 * Find a Lock Token PDA (escrow token account for a lock)
 * Seeds: ["lock_token", lock_account]
 */
export async function findLockTokenPda(
  lockAccount: Address,
  programAddress: Address = LOCKSMITH_PROGRAM_ADDRESS
): Promise<ProgramDerivedAddress> {
  const addressEncoder = getAddressEncoder();

  return getProgramDerivedAddress({
    programAddress,
    seeds: [LOCK_TOKEN_SEED, addressEncoder.encode(lockAccount)],
  });
}

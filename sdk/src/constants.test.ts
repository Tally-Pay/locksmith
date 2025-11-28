import { describe, it, expect } from "vitest";
import {
  USDC_MINT,
  FEE_USDC,
  MAX_LOCK_DURATION_SECONDS,
  CONFIG_DISCRIMINATOR,
  LOCK_DISCRIMINATOR,
} from "./constants";
import { LOCKSMITH_PROGRAM_ADDRESS } from "./generated";
import { getConfigAccountSize, getLockAccountSize } from "./generated";

/**
 * These tests validate that SDK constants match the Rust program constants.
 *
 * Rust constants from state.rs:
 *   - USDC_MINT: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" (devnet)
 *   - FEE_USDC: 150_000 (0.15 USDC)
 *   - MAX_LOCK_DURATION_SECONDS: 315_360_000 (10 years)
 *   - ConfigAccount::DISCRIMINATOR: "CONFIG\0\0"
 *   - LockAccount::DISCRIMINATOR: "LOCK\0\0\0\0"
 *   - ConfigAccount::SIZE: 41
 *   - LockAccount::SIZE: 105
 */

describe("USDC Mint constant", () => {
  it("matches the devnet USDC mint address", () => {
    // This is the devnet USDC mint used for testing
    // Mainnet USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    expect(USDC_MINT).toBe("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
  });

  it("is a valid base58 Solana address", () => {
    // Solana addresses are 32 bytes encoded in base58
    // Valid base58 chars: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
    const base58Regex = /^[1-9A-HJ-NP-Za-km-z]+$/;
    expect(USDC_MINT).toMatch(base58Regex);
    expect(USDC_MINT.length).toBeGreaterThanOrEqual(32);
    expect(USDC_MINT.length).toBeLessThanOrEqual(44);
  });
});

describe("Fee constant", () => {
  it("FEE_USDC matches Rust constant (0.15 USDC)", () => {
    // USDC has 6 decimals, so 0.15 USDC = 150,000 base units
    expect(FEE_USDC).toBe(150_000n);
  });

  it("FEE_USDC represents 0.15 USDC", () => {
    const USDC_DECIMALS = 6;
    const feeInUsdc = Number(FEE_USDC) / 10 ** USDC_DECIMALS;
    expect(feeInUsdc).toBe(0.15);
  });

  it("FEE_USDC is reasonable for a transaction fee", () => {
    // Fee should be between $0.01 and $10
    const USDC_DECIMALS = 6;
    const feeInUsdc = Number(FEE_USDC) / 10 ** USDC_DECIMALS;
    expect(feeInUsdc).toBeGreaterThanOrEqual(0.01);
    expect(feeInUsdc).toBeLessThanOrEqual(10);
  });
});

describe("Max lock duration constant", () => {
  it("matches Rust constant (10 years in seconds)", () => {
    // 10 * 365 * 24 * 60 * 60 = 315,360,000
    expect(MAX_LOCK_DURATION_SECONDS).toBe(315_360_000n);
  });

  it("is exactly 10 years without leap years", () => {
    const secondsPerMinute = 60n;
    const minutesPerHour = 60n;
    const hoursPerDay = 24n;
    const daysPerYear = 365n;
    const years = 10n;

    const expected =
      years *
      daysPerYear *
      hoursPerDay *
      minutesPerHour *
      secondsPerMinute;
    expect(MAX_LOCK_DURATION_SECONDS).toBe(expected);
  });

  it("is positive", () => {
    expect(MAX_LOCK_DURATION_SECONDS).toBeGreaterThan(0n);
  });

  it("can be added to current timestamp without overflow", () => {
    // Current time is roughly 1.7 billion seconds since epoch
    const currentTimestamp = 1_700_000_000n;
    const futureTimestamp = currentTimestamp + MAX_LOCK_DURATION_SECONDS;

    // Should not overflow i64
    const maxI64 = 9223372036854775807n;
    expect(futureTimestamp).toBeLessThan(maxI64);

    // Result should be around year 2033
    expect(futureTimestamp).toBeGreaterThan(currentTimestamp);
  });
});

describe("Program address", () => {
  it("is a valid Solana address", () => {
    const base58Regex = /^[1-9A-HJ-NP-Za-km-z]+$/;
    expect(LOCKSMITH_PROGRAM_ADDRESS).toMatch(base58Regex);
  });

  it("matches the deployed program ID", () => {
    // This should match the program ID in lib.rs: declare_id!()
    expect(LOCKSMITH_PROGRAM_ADDRESS).toBe(
      "A5vz72a5ipKUJZxmGUjGtS7uhWfzr6jhDgV2q73YhD8A"
    );
  });
});

describe("Account discriminators", () => {
  it("CONFIG_DISCRIMINATOR is correct ASCII bytes", () => {
    // "CONFIG\0\0" in ASCII
    const expected = new Uint8Array([67, 79, 78, 70, 73, 71, 0, 0]);
    expect(Array.from(CONFIG_DISCRIMINATOR)).toEqual(Array.from(expected));
  });

  it("LOCK_DISCRIMINATOR is correct ASCII bytes", () => {
    // "LOCK\0\0\0\0" in ASCII
    const expected = new Uint8Array([76, 79, 67, 75, 0, 0, 0, 0]);
    expect(Array.from(LOCK_DISCRIMINATOR)).toEqual(Array.from(expected));
  });

  it("discriminators are 8 bytes each", () => {
    expect(CONFIG_DISCRIMINATOR.length).toBe(8);
    expect(LOCK_DISCRIMINATOR.length).toBe(8);
  });
});

describe("Account sizes", () => {
  it("ConfigAccount size matches Rust (41 bytes)", () => {
    // 8 (discriminator) + 32 (admin) + 1 (bump) = 41
    expect(getConfigAccountSize()).toBe(41);
  });

  it("LockAccount size matches Rust (105 bytes)", () => {
    // 8 (discriminator) + 32 (owner) + 32 (mint) + 8 (amount)
    // + 8 (unlock_timestamp) + 8 (created_at) + 8 (lock_id) + 1 (bump) = 105
    expect(getLockAccountSize()).toBe(105);
  });

  it("ConfigAccount size breakdown is correct", () => {
    const discriminator = 8;
    const admin = 32;
    const bump = 1;
    const expected = discriminator + admin + bump;

    expect(getConfigAccountSize()).toBe(expected);
  });

  it("LockAccount size breakdown is correct", () => {
    const discriminator = 8;
    const owner = 32;
    const mint = 32;
    const amount = 8;
    const unlockTimestamp = 8;
    const createdAt = 8;
    const lockId = 8;
    const bump = 1;
    const expected =
      discriminator +
      owner +
      mint +
      amount +
      unlockTimestamp +
      createdAt +
      lockId +
      bump;

    expect(getLockAccountSize()).toBe(expected);
  });
});

describe("Cross-validation with Rust tests", () => {
  it("MAX_LOCK_DURATION_SECONDS matches test_max_lock_duration_constant", () => {
    // From Rust: assert_eq!(MAX_LOCK_DURATION_SECONDS, 315_360_000);
    expect(Number(MAX_LOCK_DURATION_SECONDS)).toBe(315_360_000);
  });

  it("FEE_USDC matches test_fee_usdc_value", () => {
    // From Rust: assert_eq!(FEE_USDC, 150_000);
    expect(Number(FEE_USDC)).toBe(150_000);
  });

  it("ConfigAccount size matches test_config_account_size", () => {
    // From Rust: assert_eq!(ConfigAccount::SIZE, 41);
    expect(getConfigAccountSize()).toBe(41);
  });

  it("LockAccount size matches test_lock_account_size", () => {
    // From Rust: assert_eq!(LockAccount::SIZE, 105);
    expect(getLockAccountSize()).toBe(105);
  });
});

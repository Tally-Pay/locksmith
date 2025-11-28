import { describe, it, expect } from "vitest";
import { createNoopSigner, type Address } from "@solana/kit";
import {
  getInitializeLockInstruction,
  getUnlockInstruction,
  getInitializeConfigInstruction,
  getTransferAdminInstruction,
  getWithdrawFeesInstruction,
  getInitializeLockInstructionDataEncoder,
  getInitializeLockInstructionDataDecoder,
  getUnlockInstructionDataEncoder,
  getUnlockInstructionDataDecoder,
  parseInitializeLockInstruction,
  parseUnlockInstruction,
  INITIALIZE_LOCK_DISCRIMINATOR,
  UNLOCK_DISCRIMINATOR,
} from "./generated";

/**
 * These tests verify that the SDK produces instruction data
 * that matches the byte layout expected by the Rust program.
 *
 * The Rust program expects:
 * - InitializeLock (tag 3): [tag:u8][amount:u64_le][unlock_timestamp:i64_le][lock_id:u64_le]
 * - Unlock (tag 4): [tag:u8][lock_id:u64_le]
 */

// Valid base58 Solana addresses for testing
const TEST_ADDRESSES = {
  owner: "7EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV" as Address,
  ownerToken: "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde" as Address,
  ownerUsdc: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" as Address,
  mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" as Address,
  lockAccount: "BPFLoaderUpgradeab1e11111111111111111111111" as Address,
  lockToken: "SysvarRent111111111111111111111111111111111" as Address,
  feeVault: "SysvarC1ock11111111111111111111111111111111" as Address,
  config: "Vote111111111111111111111111111111111111111" as Address,
  newAdmin: "Stake11111111111111111111111111111111111111" as Address,
  adminToken: "Config1111111111111111111111111111111111111" as Address,
  usdcMint: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" as Address,
};

describe("Instruction encoding", () => {
  describe("discriminators", () => {
    it("InitializeLock discriminator is 3", () => {
      expect(INITIALIZE_LOCK_DISCRIMINATOR).toBe(3);
    });

    it("Unlock discriminator is 4", () => {
      expect(UNLOCK_DISCRIMINATOR).toBe(4);
    });
  });

  describe("InitializeLock instruction", () => {
    it("encodes data in correct byte layout", () => {
      const encoder = getInitializeLockInstructionDataEncoder();
      const data = encoder.encode({
        amount: 1_000_000n,
        unlockTimestamp: 1700000000n,
        lockId: 42n,
      });

      // Expected layout:
      // [0]: discriminator (3)
      // [1-8]: amount (1_000_000 = 0x000F4240) in little-endian
      // [9-16]: unlock_timestamp (1700000000 = 0x6552DD80) in little-endian
      // [17-24]: lock_id (42 = 0x2A) in little-endian
      expect(data.length).toBe(25);
      expect(data[0]).toBe(3); // discriminator

      // Verify amount bytes (little-endian)
      const amountBytes = data.slice(1, 9);
      const amount = new DataView(amountBytes.buffer).getBigUint64(0, true);
      expect(amount).toBe(1_000_000n);

      // Verify timestamp bytes (little-endian, signed)
      const timestampBytes = data.slice(9, 17);
      const timestamp = new DataView(timestampBytes.buffer).getBigInt64(0, true);
      expect(timestamp).toBe(1700000000n);

      // Verify lock_id bytes (little-endian)
      const lockIdBytes = data.slice(17, 25);
      const lockId = new DataView(lockIdBytes.buffer).getBigUint64(0, true);
      expect(lockId).toBe(42n);
    });

    it("encodes max u64 amount correctly", () => {
      const encoder = getInitializeLockInstructionDataEncoder();
      const maxU64 = 18446744073709551615n;

      const data = encoder.encode({
        amount: maxU64,
        unlockTimestamp: 0n,
        lockId: 0n,
      });

      const amountBytes = data.slice(1, 9);
      const amount = new DataView(amountBytes.buffer).getBigUint64(0, true);
      expect(amount).toBe(maxU64);
    });

    it("encodes negative timestamp correctly", () => {
      const encoder = getInitializeLockInstructionDataEncoder();
      const negativeTimestamp = -1n;

      const data = encoder.encode({
        amount: 0n,
        unlockTimestamp: negativeTimestamp,
        lockId: 0n,
      });

      const timestampBytes = data.slice(9, 17);
      const timestamp = new DataView(timestampBytes.buffer).getBigInt64(0, true);
      expect(timestamp).toBe(-1n);
    });

    it("decodes data correctly", () => {
      const encoder = getInitializeLockInstructionDataEncoder();
      const decoder = getInitializeLockInstructionDataDecoder();

      const original = {
        amount: 1_000_000n,
        unlockTimestamp: 1700000000n,
        lockId: 42n,
      };

      const encoded = encoder.encode(original);
      const decoded = decoder.decode(encoded);

      expect(decoded.discriminator).toBe(3);
      expect(decoded.amount).toBe(original.amount);
      expect(decoded.unlockTimestamp).toBe(original.unlockTimestamp);
      expect(decoded.lockId).toBe(original.lockId);
    });

    it("produces data matching Rust little-endian test case", () => {
      // This test mirrors the Rust test: test_unpack_initialize_lock_little_endian
      const encoder = getInitializeLockInstructionDataEncoder();
      const data = encoder.encode({
        amount: 0x0102030405060708n,
        unlockTimestamp: BigInt("0x090A0B0C0D0E0F10"),
        lockId: 0x1112131415161718n,
      });

      // Rust expects these exact bytes (little-endian):
      const expectedBytes = new Uint8Array([
        3, // tag
        0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // amount (little-endian)
        0x10, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, // timestamp (little-endian)
        0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11, // lock_id (little-endian)
      ]);

      expect(Array.from(data)).toEqual(Array.from(expectedBytes));
    });
  });

  describe("Unlock instruction", () => {
    it("encodes data in correct byte layout", () => {
      const encoder = getUnlockInstructionDataEncoder();
      const data = encoder.encode({ lockId: 42n });

      // Expected layout:
      // [0]: discriminator (4)
      // [1-8]: lock_id (42) in little-endian
      expect(data.length).toBe(9);
      expect(data[0]).toBe(4); // discriminator

      const lockIdBytes = data.slice(1, 9);
      const lockId = new DataView(lockIdBytes.buffer).getBigUint64(0, true);
      expect(lockId).toBe(42n);
    });

    it("encodes max u64 lock_id correctly", () => {
      const encoder = getUnlockInstructionDataEncoder();
      const maxU64 = 18446744073709551615n;

      const data = encoder.encode({ lockId: maxU64 });

      const lockIdBytes = data.slice(1, 9);
      const lockId = new DataView(lockIdBytes.buffer).getBigUint64(0, true);
      expect(lockId).toBe(maxU64);
    });

    it("encodes zero lock_id correctly", () => {
      const encoder = getUnlockInstructionDataEncoder();

      const data = encoder.encode({ lockId: 0n });

      const lockIdBytes = data.slice(1, 9);
      const lockId = new DataView(lockIdBytes.buffer).getBigUint64(0, true);
      expect(lockId).toBe(0n);
    });

    it("decodes data correctly", () => {
      const encoder = getUnlockInstructionDataEncoder();
      const decoder = getUnlockInstructionDataDecoder();

      const original = { lockId: 999n };
      const encoded = encoder.encode(original);
      const decoded = decoder.decode(encoded);

      expect(decoded.discriminator).toBe(4);
      expect(decoded.lockId).toBe(original.lockId);
    });
  });

  describe("Full instruction creation", () => {
    const ownerSigner = createNoopSigner(TEST_ADDRESSES.owner);

    it("creates InitializeLock instruction with all accounts", () => {
      const instruction = getInitializeLockInstruction({
        owner: ownerSigner,
        ownerTokenAccount: TEST_ADDRESSES.ownerToken,
        ownerUsdcAccount: TEST_ADDRESSES.ownerUsdc,
        mint: TEST_ADDRESSES.mint,
        lockAccount: TEST_ADDRESSES.lockAccount,
        lockTokenAccount: TEST_ADDRESSES.lockToken,
        feeVault: TEST_ADDRESSES.feeVault,
        amount: 1_000_000n,
        unlockTimestamp: 1700000000n,
        lockId: 1n,
      });

      expect(instruction.accounts.length).toBe(9);
      expect(instruction.data.length).toBe(25);
      expect(instruction.data[0]).toBe(3); // InitializeLock discriminator
    });

    it("creates Unlock instruction with all accounts", () => {
      const instruction = getUnlockInstruction({
        owner: ownerSigner,
        ownerTokenAccount: TEST_ADDRESSES.ownerToken,
        lockAccount: TEST_ADDRESSES.lockAccount,
        lockTokenAccount: TEST_ADDRESSES.lockToken,
        lockId: 42n,
      });

      expect(instruction.accounts.length).toBe(5);
      expect(instruction.data.length).toBe(9);
      expect(instruction.data[0]).toBe(4); // Unlock discriminator
    });

    it("creates InitializeConfig instruction", () => {
      const instruction = getInitializeConfigInstruction({
        admin: ownerSigner,
        config: TEST_ADDRESSES.config,
        usdcMint: TEST_ADDRESSES.usdcMint,
        feeVault: TEST_ADDRESSES.feeVault,
      });

      expect(instruction.accounts.length).toBe(6);
      expect(instruction.data[0]).toBe(0); // InitializeConfig discriminator
    });

    it("creates TransferAdmin instruction", () => {
      const instruction = getTransferAdminInstruction({
        admin: ownerSigner,
        newAdmin: TEST_ADDRESSES.newAdmin,
        config: TEST_ADDRESSES.config,
      });

      expect(instruction.accounts.length).toBe(3);
      expect(instruction.data[0]).toBe(1); // TransferAdmin discriminator
    });

    it("creates WithdrawFees instruction", () => {
      const instruction = getWithdrawFeesInstruction({
        admin: ownerSigner,
        config: TEST_ADDRESSES.config,
        feeVault: TEST_ADDRESSES.feeVault,
        adminTokenAccount: TEST_ADDRESSES.adminToken,
      });

      expect(instruction.accounts.length).toBe(5);
      expect(instruction.data[0]).toBe(2); // WithdrawFees discriminator
    });

    it("accepts number type for bigint fields", () => {
      // TypeScript allows number | bigint for these fields
      const instruction = getInitializeLockInstruction({
        owner: ownerSigner,
        ownerTokenAccount: TEST_ADDRESSES.ownerToken,
        ownerUsdcAccount: TEST_ADDRESSES.ownerUsdc,
        mint: TEST_ADDRESSES.mint,
        lockAccount: TEST_ADDRESSES.lockAccount,
        lockTokenAccount: TEST_ADDRESSES.lockToken,
        feeVault: TEST_ADDRESSES.feeVault,
        amount: 1000000, // number instead of bigint
        unlockTimestamp: 1700000000,
        lockId: 1,
      });

      expect(instruction.data[0]).toBe(3);
    });
  });

  describe("Instruction parsing", () => {
    const ownerSigner = createNoopSigner(TEST_ADDRESSES.owner);

    it("parses InitializeLock instruction correctly", () => {
      const instruction = getInitializeLockInstruction({
        owner: ownerSigner,
        ownerTokenAccount: TEST_ADDRESSES.ownerToken,
        ownerUsdcAccount: TEST_ADDRESSES.ownerUsdc,
        mint: TEST_ADDRESSES.mint,
        lockAccount: TEST_ADDRESSES.lockAccount,
        lockTokenAccount: TEST_ADDRESSES.lockToken,
        feeVault: TEST_ADDRESSES.feeVault,
        amount: 1_000_000n,
        unlockTimestamp: 1700000000n,
        lockId: 42n,
      });

      const parsed = parseInitializeLockInstruction(instruction);

      expect(parsed.data.discriminator).toBe(3);
      expect(parsed.data.amount).toBe(1_000_000n);
      expect(parsed.data.unlockTimestamp).toBe(1700000000n);
      expect(parsed.data.lockId).toBe(42n);
      expect(parsed.accounts.owner).toBeDefined();
      expect(parsed.accounts.mint).toBeDefined();
    });

    it("parses Unlock instruction correctly", () => {
      const instruction = getUnlockInstruction({
        owner: ownerSigner,
        ownerTokenAccount: TEST_ADDRESSES.ownerToken,
        lockAccount: TEST_ADDRESSES.lockAccount,
        lockTokenAccount: TEST_ADDRESSES.lockToken,
        lockId: 42n,
      });

      const parsed = parseUnlockInstruction(instruction);

      expect(parsed.data.discriminator).toBe(4);
      expect(parsed.data.lockId).toBe(42n);
      expect(parsed.accounts.owner).toBeDefined();
      expect(parsed.accounts.lockAccount).toBeDefined();
    });
  });
});

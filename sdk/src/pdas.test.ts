import { describe, it, expect } from "vitest";
import { address, type Address } from "@solana/kit";
import {
  findConfigPda,
  findFeeVaultPda,
  findLockAccountPda,
  findLockTokenPda,
} from "./pdas";
import { LOCKSMITH_PROGRAM_ADDRESS } from "./generated";

// Valid base58 Solana addresses for testing (these are real valid addresses)
const TEST_ADDRESSES = {
  systemProgram: "11111111111111111111111111111111" as Address,
  tokenProgram: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as Address,
  owner1: "7EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV" as Address,
  owner2: "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde" as Address,
  mint1: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" as Address,
  mint2: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" as Address,
  lock1: "BPFLoaderUpgradeab1e11111111111111111111111" as Address,
  lock2: "SysvarRent111111111111111111111111111111111" as Address,
};

describe("PDA derivations", () => {
  describe("findConfigPda", () => {
    it("derives deterministically for the same program", async () => {
      const [pda1, bump1] = await findConfigPda();
      const [pda2, bump2] = await findConfigPda();

      expect(pda1).toBe(pda2);
      expect(bump1).toBe(bump2);
    });

    it("derives different PDAs for different programs", async () => {
      const [defaultPda] = await findConfigPda();
      const [otherPda] = await findConfigPda(TEST_ADDRESSES.systemProgram);

      expect(defaultPda).not.toBe(otherPda);
    });

    it("bump is valid (0-255)", async () => {
      const [, bump] = await findConfigPda();
      expect(bump).toBeGreaterThanOrEqual(0);
      expect(bump).toBeLessThanOrEqual(255);
    });
  });

  describe("findFeeVaultPda", () => {
    it("derives deterministically", async () => {
      const [pda1, bump1] = await findFeeVaultPda();
      const [pda2, bump2] = await findFeeVaultPda();

      expect(pda1).toBe(pda2);
      expect(bump1).toBe(bump2);
    });

    it("is different from config PDA", async () => {
      const [configPda] = await findConfigPda();
      const [feeVaultPda] = await findFeeVaultPda();

      expect(configPda).not.toBe(feeVaultPda);
    });
  });

  describe("findLockAccountPda", () => {
    const owner = TEST_ADDRESSES.owner1;
    const mint = TEST_ADDRESSES.mint1;

    it("derives deterministically for same inputs", async () => {
      const [pda1, bump1] = await findLockAccountPda(owner, mint, 0n);
      const [pda2, bump2] = await findLockAccountPda(owner, mint, 0n);

      expect(pda1).toBe(pda2);
      expect(bump1).toBe(bump2);
    });

    it("derives different PDAs for different owners", async () => {
      const [pda1] = await findLockAccountPda(owner, mint, 0n);
      const [pda2] = await findLockAccountPda(TEST_ADDRESSES.owner2, mint, 0n);

      expect(pda1).not.toBe(pda2);
    });

    it("derives different PDAs for different mints", async () => {
      const [pda1] = await findLockAccountPda(owner, mint, 0n);
      const [pda2] = await findLockAccountPda(owner, TEST_ADDRESSES.mint2, 0n);

      expect(pda1).not.toBe(pda2);
    });

    it("derives different PDAs for different lock IDs", async () => {
      const [pda1] = await findLockAccountPda(owner, mint, 0n);
      const [pda2] = await findLockAccountPda(owner, mint, 1n);
      const [pda3] = await findLockAccountPda(owner, mint, 999n);

      expect(pda1).not.toBe(pda2);
      expect(pda2).not.toBe(pda3);
      expect(pda1).not.toBe(pda3);
    });

    it("accepts both bigint and number for lockId", async () => {
      const [pda1] = await findLockAccountPda(owner, mint, 42n);
      const [pda2] = await findLockAccountPda(owner, mint, 42);

      expect(pda1).toBe(pda2);
    });

    it("handles max u64 lock ID", async () => {
      const maxU64 = 18446744073709551615n;
      const [pda, bump] = await findLockAccountPda(owner, mint, maxU64);

      expect(pda).toBeDefined();
      expect(bump).toBeGreaterThanOrEqual(0);
      expect(bump).toBeLessThanOrEqual(255);
    });

    it("uses the default program address", async () => {
      const [pda1] = await findLockAccountPda(owner, mint, 0n);
      const [pda2] = await findLockAccountPda(
        owner,
        mint,
        0n,
        LOCKSMITH_PROGRAM_ADDRESS
      );

      expect(pda1).toBe(pda2);
    });
  });

  describe("findLockTokenPda", () => {
    it("derives deterministically for same lock account", async () => {
      const lockAccount = TEST_ADDRESSES.lock1;

      const [pda1, bump1] = await findLockTokenPda(lockAccount);
      const [pda2, bump2] = await findLockTokenPda(lockAccount);

      expect(pda1).toBe(pda2);
      expect(bump1).toBe(bump2);
    });

    it("derives different PDAs for different lock accounts", async () => {
      const [pda1] = await findLockTokenPda(TEST_ADDRESSES.lock1);
      const [pda2] = await findLockTokenPda(TEST_ADDRESSES.lock2);

      expect(pda1).not.toBe(pda2);
    });

    it("is tied to lock account PDA chain", async () => {
      // Given an owner, mint, and lockId, the lock token PDA should be
      // deterministically derivable through the lock account PDA
      const owner = TEST_ADDRESSES.owner1;
      const mint = TEST_ADDRESSES.mint1;
      const lockId = 42n;

      const [lockAccountPda] = await findLockAccountPda(owner, mint, lockId);
      const [lockTokenPda] = await findLockTokenPda(lockAccountPda);

      // Verify consistency - same derivation path produces same result
      const [lockAccountPda2] = await findLockAccountPda(owner, mint, lockId);
      const [lockTokenPda2] = await findLockTokenPda(lockAccountPda2);

      expect(lockTokenPda).toBe(lockTokenPda2);
    });
  });

  describe("PDA uniqueness across types", () => {
    it("config, fee vault, and lock PDAs are all distinct", async () => {
      const [configPda] = await findConfigPda();
      const [feeVaultPda] = await findFeeVaultPda();
      const [lockAccountPda] = await findLockAccountPda(
        TEST_ADDRESSES.owner1,
        TEST_ADDRESSES.mint1,
        0n
      );
      const [lockTokenPda] = await findLockTokenPda(lockAccountPda);

      const pdas = [configPda, feeVaultPda, lockAccountPda, lockTokenPda];
      const uniquePdas = new Set(pdas);

      expect(uniquePdas.size).toBe(pdas.length);
    });
  });
});

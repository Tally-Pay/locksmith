import { describe, it, expect } from "vitest";
import { type Address, lamports } from "@solana/kit";
import {
  getConfigAccountDecoder,
  getConfigAccountEncoder,
  getConfigAccountSize,
  getLockAccountDecoder,
  getLockAccountEncoder,
  getLockAccountSize,
  decodeConfigAccount,
  decodeLockAccount,
  type ConfigAccount,
  type LockAccount,
} from "./generated";
import { CONFIG_DISCRIMINATOR, LOCK_DISCRIMINATOR } from "./constants";

// Valid base58 Solana addresses for testing
const TEST_ADDRESSES = {
  admin: "7EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV" as Address,
  owner: "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde" as Address,
  mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" as Address,
  account: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" as Address,
  program: "BPFLoaderUpgradeab1e11111111111111111111111" as Address,
};

/**
 * These tests verify that the SDK can correctly decode account data
 * that was serialized by the Rust program.
 *
 * Account layouts from Rust:
 *
 * ConfigAccount (41 bytes):
 *   [0-7]:   discriminator "CONFIG\0\0" (8 bytes)
 *   [8-39]:  admin pubkey (32 bytes)
 *   [40]:    bump (1 byte)
 *
 * LockAccount (105 bytes):
 *   [0-7]:   discriminator "LOCK\0\0\0\0" (8 bytes)
 *   [8-39]:  owner pubkey (32 bytes)
 *   [40-71]: mint pubkey (32 bytes)
 *   [72-79]: amount (u64 little-endian, 8 bytes)
 *   [80-87]: unlock_timestamp (i64 little-endian, 8 bytes)
 *   [88-95]: created_at (i64 little-endian, 8 bytes)
 *   [96-103]: lock_id (u64 little-endian, 8 bytes)
 *   [104]:   bump (1 byte)
 */

describe("Account sizes", () => {
  it("ConfigAccount size matches Rust constant", () => {
    // Rust: pub const SIZE: usize = 8 + 32 + 1 = 41
    expect(getConfigAccountSize()).toBe(41);
  });

  it("LockAccount size matches Rust constant", () => {
    // Rust: pub const SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1 = 105
    expect(getLockAccountSize()).toBe(105);
  });
});

describe("ConfigAccount decoding", () => {
  it("decodes Rust-serialized data correctly", () => {
    // Simulate data as serialized by Rust
    const adminBytes = new Uint8Array(32);
    adminBytes.fill(0x42); // Fill with recognizable pattern

    const data = new Uint8Array(41);
    // Discriminator: "CONFIG\0\0"
    data.set(new TextEncoder().encode("CONFIG\0\0"), 0);
    // Admin pubkey
    data.set(adminBytes, 8);
    // Bump
    data[40] = 255;

    const decoder = getConfigAccountDecoder();
    const decoded = decoder.decode(data);

    expect(Array.from(decoded.discriminator)).toEqual(
      Array.from(CONFIG_DISCRIMINATOR)
    );
    expect(decoded.bump).toBe(255);
  });

  it("decodes min bump value", () => {
    const data = new Uint8Array(41);
    data.set(new TextEncoder().encode("CONFIG\0\0"), 0);
    data.set(new Uint8Array(32), 8); // zero admin
    data[40] = 0;

    const decoder = getConfigAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.bump).toBe(0);
  });

  it("encode/decode roundtrip preserves data", () => {
    const encoder = getConfigAccountEncoder();
    const decoder = getConfigAccountDecoder();

    const original: ConfigAccount = {
      discriminator: CONFIG_DISCRIMINATOR,
      admin: TEST_ADDRESSES.admin,
      bump: 200,
    };

    const encoded = encoder.encode(original);
    const decoded = decoder.decode(encoded);

    expect(decoded.bump).toBe(original.bump);
    expect(decoded.admin).toBe(original.admin);
  });

  it("matches Rust byte layout exactly", () => {
    // Test case from Rust: test_config_account_byte_layout
    const adminBytes = new Uint8Array([
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
      22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    ]);

    // Build expected data
    const expectedData = new Uint8Array(41);
    expectedData.set(new TextEncoder().encode("CONFIG\0\0"), 0);
    expectedData.set(adminBytes, 8);
    expectedData[40] = 200;

    const decoder = getConfigAccountDecoder();
    const decoded = decoder.decode(expectedData);

    expect(decoded.bump).toBe(200);
    expect(Array.from(decoded.discriminator)).toEqual([
      67, 79, 78, 70, 73, 71, 0, 0,
    ]); // "CONFIG\0\0"
  });
});

describe("LockAccount decoding", () => {
  const createLockAccountData = (params: {
    owner?: Uint8Array;
    mint?: Uint8Array;
    amount?: bigint;
    unlockTimestamp?: bigint;
    createdAt?: bigint;
    lockId?: bigint;
    bump?: number;
  }): Uint8Array => {
    const data = new Uint8Array(105);

    // Discriminator: "LOCK\0\0\0\0"
    data.set(new TextEncoder().encode("LOCK\0\0\0\0"), 0);

    // Owner (32 bytes)
    data.set(params.owner || new Uint8Array(32), 8);

    // Mint (32 bytes)
    data.set(params.mint || new Uint8Array(32), 40);

    // Amount (8 bytes, little-endian)
    const amountView = new DataView(data.buffer, 72, 8);
    amountView.setBigUint64(0, params.amount || 0n, true);

    // Unlock timestamp (8 bytes, little-endian, signed)
    const timestampView = new DataView(data.buffer, 80, 8);
    timestampView.setBigInt64(0, params.unlockTimestamp || 0n, true);

    // Created at (8 bytes, little-endian, signed)
    const createdAtView = new DataView(data.buffer, 88, 8);
    createdAtView.setBigInt64(0, params.createdAt || 0n, true);

    // Lock ID (8 bytes, little-endian)
    const lockIdView = new DataView(data.buffer, 96, 8);
    lockIdView.setBigUint64(0, params.lockId || 0n, true);

    // Bump (1 byte)
    data[104] = params.bump || 0;

    return data;
  };

  it("decodes Rust-serialized data correctly", () => {
    const data = createLockAccountData({
      amount: 1_000_000_000n,
      unlockTimestamp: 1700000000n,
      createdAt: 1699000000n,
      lockId: 42n,
      bump: 254,
    });

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(Array.from(decoded.discriminator)).toEqual(
      Array.from(LOCK_DISCRIMINATOR)
    );
    expect(decoded.amount).toBe(1_000_000_000n);
    expect(decoded.unlockTimestamp).toBe(1700000000n);
    expect(decoded.createdAt).toBe(1699000000n);
    expect(decoded.lockId).toBe(42n);
    expect(decoded.bump).toBe(254);
  });

  it("decodes max u64 amount", () => {
    const maxU64 = 18446744073709551615n;
    const data = createLockAccountData({ amount: maxU64 });

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.amount).toBe(maxU64);
  });

  it("decodes max i64 timestamp", () => {
    const maxI64 = 9223372036854775807n;
    const data = createLockAccountData({ unlockTimestamp: maxI64 });

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.unlockTimestamp).toBe(maxI64);
  });

  it("decodes min i64 (negative) timestamp", () => {
    const minI64 = -9223372036854775808n;
    const data = createLockAccountData({ unlockTimestamp: minI64 });

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.unlockTimestamp).toBe(minI64);
  });

  it("decodes negative timestamps correctly", () => {
    const data = createLockAccountData({ unlockTimestamp: -1n });

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.unlockTimestamp).toBe(-1n);
  });

  it("decodes max lock_id", () => {
    const maxU64 = 18446744073709551615n;
    const data = createLockAccountData({ lockId: maxU64 });

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.lockId).toBe(maxU64);
  });

  it("decodes zero amount correctly", () => {
    const data = createLockAccountData({ amount: 0n });

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.amount).toBe(0n);
  });

  it("encode/decode roundtrip preserves data", () => {
    const encoder = getLockAccountEncoder();
    const decoder = getLockAccountDecoder();

    const original: LockAccount = {
      discriminator: LOCK_DISCRIMINATOR,
      owner: TEST_ADDRESSES.owner,
      mint: TEST_ADDRESSES.mint,
      amount: 1_000_000_000n,
      unlockTimestamp: 1700000000n,
      createdAt: 1699000000n,
      lockId: 42n,
      bump: 254,
    };

    const encoded = encoder.encode(original);
    const decoded = decoder.decode(encoded);

    expect(decoded.owner).toBe(original.owner);
    expect(decoded.mint).toBe(original.mint);
    expect(decoded.amount).toBe(original.amount);
    expect(decoded.unlockTimestamp).toBe(original.unlockTimestamp);
    expect(decoded.createdAt).toBe(original.createdAt);
    expect(decoded.lockId).toBe(original.lockId);
    expect(decoded.bump).toBe(original.bump);
  });

  it("matches Rust byte layout from test_lock_account_byte_layout", () => {
    // Exact test case from Rust
    const ownerBytes = new Uint8Array(32).fill(1);
    const mintBytes = new Uint8Array(32).fill(2);

    const data = new Uint8Array(105);
    // Discriminator
    data.set(new TextEncoder().encode("LOCK\0\0\0\0"), 0);
    // Owner
    data.set(ownerBytes, 8);
    // Mint
    data.set(mintBytes, 40);
    // Amount: 0x0102030405060708 in little-endian
    data.set([0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01], 72);
    // Unlock timestamp: 0x090A0B0C0D0E0F10 in little-endian
    data.set([0x10, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09], 80);
    // Created at: 0x1112131415161718 in little-endian
    data.set([0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11], 88);
    // Lock ID: 0x191A1B1C1D1E1F20 in little-endian
    data.set([0x20, 0x1f, 0x1e, 0x1d, 0x1c, 0x1b, 0x1a, 0x19], 96);
    // Bump
    data[104] = 250;

    const decoder = getLockAccountDecoder();
    const decoded = decoder.decode(data);

    expect(decoded.amount).toBe(0x0102030405060708n);
    expect(decoded.unlockTimestamp).toBe(BigInt("0x090A0B0C0D0E0F10"));
    expect(decoded.createdAt).toBe(BigInt("0x1112131415161718"));
    expect(decoded.lockId).toBe(0x191a1b1c1d1e1f20n);
    expect(decoded.bump).toBe(250);
  });
});

describe("Discriminator validation", () => {
  it("CONFIG discriminator is 'CONFIG\\0\\0' in bytes", () => {
    const expected = new TextEncoder().encode("CONFIG\0\0");
    expect(Array.from(CONFIG_DISCRIMINATOR)).toEqual(Array.from(expected));
  });

  it("LOCK discriminator is 'LOCK\\0\\0\\0\\0' in bytes", () => {
    const expected = new TextEncoder().encode("LOCK\0\0\0\0");
    expect(Array.from(LOCK_DISCRIMINATOR)).toEqual(Array.from(expected));
  });

  it("discriminators are exactly 8 bytes", () => {
    expect(CONFIG_DISCRIMINATOR.length).toBe(8);
    expect(LOCK_DISCRIMINATOR.length).toBe(8);
  });

  it("discriminators are unique", () => {
    expect(Array.from(CONFIG_DISCRIMINATOR)).not.toEqual(
      Array.from(LOCK_DISCRIMINATOR)
    );
  });
});

describe("decodeAccount helpers", () => {
  it("decodeConfigAccount handles MaybeEncodedAccount format", () => {
    const data = new Uint8Array(41);
    data.set(new TextEncoder().encode("CONFIG\0\0"), 0);
    data.set(new Uint8Array(32), 8);
    data[40] = 123;

    const encodedAccount = {
      exists: true as const,
      address: TEST_ADDRESSES.account,
      data,
      executable: false,
      lamports: lamports(1000000n),
      programAddress: TEST_ADDRESSES.program,
      space: BigInt(data.length),
    };

    const decoded = decodeConfigAccount(encodedAccount);

    expect(decoded.data.bump).toBe(123);
  });

  it("decodeLockAccount handles MaybeEncodedAccount format", () => {
    const data = new Uint8Array(105);
    data.set(new TextEncoder().encode("LOCK\0\0\0\0"), 0);
    data.set(new Uint8Array(32), 8); // owner
    data.set(new Uint8Array(32), 40); // mint
    // Set amount to 1000
    new DataView(data.buffer, 72, 8).setBigUint64(0, 1000n, true);
    data[104] = 200;

    const encodedAccount = {
      exists: true as const,
      address: TEST_ADDRESSES.account,
      data,
      executable: false,
      lamports: lamports(1000000n),
      programAddress: TEST_ADDRESSES.program,
      space: BigInt(data.length),
    };

    const decoded = decodeLockAccount(encodedAccount);

    expect(decoded.data.amount).toBe(1000n);
    expect(decoded.data.bump).toBe(200);
  });

  it("handles non-existent accounts", () => {
    const nonExistent = {
      exists: false as const,
      address: TEST_ADDRESSES.account,
    };

    const decoded = decodeConfigAccount(nonExistent);
    expect(decoded.exists).toBe(false);
  });
});

import type { Address } from "@solana/kit";

/**
 * USDC mint address (devnet)
 */
export const USDC_MINT =
  "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" as Address<"4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU">;

/**
 * Fee amount: 0.15 USDC (USDC has 6 decimals)
 */
export const FEE_USDC = 150_000n;

/**
 * Maximum lock duration: 10 years in seconds
 */
export const MAX_LOCK_DURATION_SECONDS = 10n * 365n * 24n * 60n * 60n;

/**
 * ConfigAccount discriminator bytes
 */
export const CONFIG_DISCRIMINATOR = new Uint8Array([
  67, 79, 78, 70, 73, 71, 0, 0,
]); // "CONFIG\0\0"

/**
 * LockAccount discriminator bytes
 */
export const LOCK_DISCRIMINATOR = new Uint8Array([76, 79, 67, 75, 0, 0, 0, 0]); // "LOCK\0\0\0\0"

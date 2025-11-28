/**
 * Locksmith SDK
 *
 * Type-safe SDK for interacting with the Locksmith token locking program.
 *
 * @example
 * ```typescript
 * import {
 *   getInitializeLockInstruction,
 *   findLockAccountPda,
 *   findLockTokenPda,
 *   LOCKSMITH_PROGRAM_ADDRESS,
 *   FEE_USDC,
 * } from "@locksmith/sdk";
 * ```
 */

// Re-export all generated types
export * from "./generated";

// Export PDA helpers and constants
export * from "./pdas";
export * from "./constants";

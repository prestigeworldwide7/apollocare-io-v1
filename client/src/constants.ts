import { PublicKey } from "@solana/web3.js";

// Program ID as defined in the Anchor program (see programs/apollo_core/src/lib.rs)
export const PROGRAM_ID = new PublicKey("Apoll1CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCcApH");

// Seeds used by the program.  These constants mirror the seeds defined in
// the Rust program.  They are used to derive PDAs for the config account
// and program token accounts.  Anchor’s `findProgramAddress` should be used
// to compute these addresses.
export const CONFIG_SEED = Buffer.from("config");
export const PREMIUM_POOL_SEED = Buffer.from("premium_pool");
export const CAPITAL_POOL_SEED = Buffer.from("capital_pool");

/**
 * Derives the config PDA for the Apollo program.
 */
export function getConfigPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([CONFIG_SEED], PROGRAM_ID);
}

/**
 * Derives the premium pool PDA.  The pool is owned by the config PDA and
 * holds USDC for claims.  Requires the USDC mint address to derive the
 * associated token account (ATA).
 */
export function getPremiumPoolPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([PREMIUM_POOL_SEED], PROGRAM_ID);
}

/**
 * Derives the capital pool PDA.  Holds APH staked by members.
 */
export function getCapitalPoolPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([CAPITAL_POOL_SEED], PROGRAM_ID);
}

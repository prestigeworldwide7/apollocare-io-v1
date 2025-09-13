/*
 * Example script to submit a claim.  It calls the `submit_claim` instruction
 * on the Apollo program.  Provide the claim amount and a 32‑byte hash of
 * the off‑chain documentation (for example the SHA‑256 hash of an invoice).
 */

import * as anchor from '@coral-xyz/anchor';
import { Program, Wallet, web3 } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import idl from '../target/idl/apollo_core.json';
import { PROGRAM_ID, getConfigPda, getPremiumPoolPda } from './constants';

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = new Program(idl as anchor.Idl, PROGRAM_ID, provider);
  const wallet = provider.wallet as Wallet;
  const user = wallet.payer as web3.Keypair;

  // Replace with the policy address and USDC mint used in your deployment
  const policyPubkey = new PublicKey('ReplaceWithPolicyAddress');
  const usdcMint = new PublicKey('ReplaceWithUsdCMint');
  const amount = 100_000; // Claim amount (e.g. 0.1 USDC with 6 decimals)
  // Provide a 32‑byte hash.  For testing you can use an array of zeros.
  const offchainHash = new Uint8Array(32);

  const [configPda] = getConfigPda();
  const [memberPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('member'), user.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const [premiumPoolPda] = getPremiumPoolPda();
  const [claimPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('claim'), memberPda.toBuffer(), Buffer.from([0, 0, 0, 0, 0, 0, 0, 0])],
    PROGRAM_ID,
  );
  const userUsdc = await getAssociatedTokenAddress(usdcMint, user.publicKey);

  const txSig = await program.methods
    .submitClaim(new anchor.BN(amount), Array.from(offchainHash))
    .accounts({
      config: configPda,
      member: memberPda,
      authority: user.publicKey,
      policy: policyPubkey,
      claim: claimPda,
      premiumPool: premiumPoolPda,
      userUsdcAccount: userUsdc,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([])
    .rpc();
  console.log('Claim submitted, tx:', txSig);
}

main().catch((err) => {
  console.error(err);
});

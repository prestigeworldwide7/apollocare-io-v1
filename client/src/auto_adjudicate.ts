/*
 * AI-assisted claim adjudication script.
 *
 * This script demonstrates how an off-chain AI agent could be used to
 * automatically evaluate and approve claims.  The on-chain program does not
 * perform any machine learning inference; instead, this script calls a
 * placeholder function `evaluateClaimWithAI` that should be replaced with
 * your AI service integration.  If the AI agent approves the claim, the
 * script will invoke the `approveClaim` instruction on the Apollo program.
 */

import * as anchor from '@coral-xyz/anchor';
import { Program, Wallet } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import idl from '../target/idl/apollo_core.json';
import { PROGRAM_ID, getConfigPda, getPremiumPoolPda } from './constants';

/**
 * Placeholder for AI model inference.
 * In production, replace this stub with a call to your AI service using the
 * claim metadata and off-chain hashed documents.  Return `true` to approve
 * the claim, or `false` to decline.
 */
async function evaluateClaimWithAI(claim: PublicKey): Promise<boolean> {
  console.log('Evaluating claim', claim.toBase58(), 'with AI agent...');
  // TODO: call external AI API or model here
  // For this example we always approve the claim.
  return true;
}

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = new Program(idl as anchor.Idl, PROGRAM_ID, provider);
  const wallet = provider.wallet as Wallet;

  // Replace with the claim PDA you wish to evaluate
  const claimPda = new PublicKey('ReplaceWithClaimPda');
  // Replace with the member PDA and claimant USDC ATA addresses
  const memberPda = new PublicKey('ReplaceWithMemberPda');
  const userUsdc = new PublicKey('ReplaceWithClaimantUsdcAta');

  const [configPda] = getConfigPda();
  const [premiumPoolPda] = getPremiumPoolPda();

  const ok = await evaluateClaimWithAI(claimPda);
  if (!ok) {
    console.log('AI agent recommends denying the claim.');
    return;
  }

  console.log('AI agent approved claim; sending transaction...');
  const txSig = await program.methods
    .approveClaim()
    .accounts({
      config: configPda,
      member: memberPda,
      claim: claimPda,
      authority: wallet.publicKey,
      premiumPool: premiumPoolPda,
      userUsdcAccount: userUsdc,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([])
    .rpc();
  console.log('Claim approved on-chain, tx:', txSig);
}

main().catch((err) => console.error(err));

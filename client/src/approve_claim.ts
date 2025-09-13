/*
 * Approves a pending claim.  Only the protocol authority should run this
 * script.  Provide the claim PDA address as a parameter.
 */

import * as anchor from '@coral-xyz/anchor';
import { Program, Wallet } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import idl from '../target/idl/apollo_core.json';
import { PROGRAM_ID, getConfigPda, getPremiumPoolPda } from './constants';

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = new Program(idl as anchor.Idl, PROGRAM_ID, provider);
  const wallet = provider.wallet as Wallet;

  // Replace with the claim PDA you wish to approve
  const claimPda = new PublicKey('ReplaceWithClaimPda');
  // Replace with the member's authority (the claimant) and their USDC ATA
  const claimant = new PublicKey('ReplaceWithClaimantPubkey');
  const userUsdc = new PublicKey('ReplaceWithClaimantUsdcAta');

  const [configPda] = getConfigPda();
  const [premiumPoolPda] = getPremiumPoolPda();
  // The member PDA can be derived if needed, but is not required here.
  const txSig = await program.methods
    .approveClaim()
    .accounts({
      config: configPda,
      member: new PublicKey('ReplaceWithMemberPda'),
      claim: claimPda,
      authority: wallet.publicKey,
      premiumPool: premiumPoolPda,
      userUsdcAccount: userUsdc,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([])
    .rpc();
  console.log('Approved claim tx:', txSig);
}

main().catch((err) => console.error(err));

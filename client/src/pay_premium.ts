/*
 * Example script to pay an additional premium.  This script calls the
 * `pay_premium` instruction.  It assumes that the user is already
 * enrolled in the policy and has sufficient USDC.  See enroll.ts for
 * notes on configuration.
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

  // Replace with the policy address used during enrollment
  const policyPubkey = new PublicKey('ReplaceWithPolicyAddress');
  const usdcMint = new PublicKey('ReplaceWithUsdCMint');
  const [configPda] = getConfigPda();
  const [premiumPoolPda] = getPremiumPoolPda();
  const userUsdc = await getAssociatedTokenAddress(usdcMint, user.publicKey);

  const txSig = await program.methods
    .payPremium()
    .accounts({
      config: configPda,
      policy: policyPubkey,
      authority: user.publicKey,
      userUsdcAccount: userUsdc,
      premiumPool: premiumPoolPda,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([])
    .rpc();
  console.log('Premium payment tx:', txSig);
}

main().catch((err) => {
  console.error(err);
});

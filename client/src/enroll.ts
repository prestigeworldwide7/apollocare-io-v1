/*
 * Example script to enroll a user into a policy.  This script uses the
 * Anchor client to invoke the `enroll_member` instruction.  It expects the
 * ApolloCore program to be deployed locally and the IDL to be available at
 * `../target/idl/apollo_core.json`.  Before running, deploy the program
 * using `anchor build && anchor deploy` and update the policy address.
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

  // Replace with the actual policy address created via `anchor create-policy`
  const policyPubkey = new PublicKey('ReplaceWithPolicyAddress');

  // Derive PDAs from the program seeds
  const [configPda] = getConfigPda();
  const [premiumPoolPda] = getPremiumPoolPda();
  const [memberPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('member'), user.publicKey.toBuffer()],
    PROGRAM_ID,
  );

  // Determine the user's USDC associated token account.  Replace the USDC
  // mint address with the one used in your deployment.  If the account does
  // not exist, it must be created separately.
  const usdcMint = new PublicKey('ReplaceWithUsdCMint');
  const userUsdc = await getAssociatedTokenAddress(usdcMint, user.publicKey);

  const txSig = await program.methods
    .enrollMember()
    .accounts({
      config: configPda,
      policy: policyPubkey,
      member: memberPda,
      authority: user.publicKey,
      userUsdcAccount: userUsdc,
      premiumPool: premiumPoolPda,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([])
    .rpc();
  console.log('Enrollment transaction signature:', txSig);
}

main().catch((err) => {
  console.error(err);
});

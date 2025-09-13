/*
 * Example script to stake APH tokens into the capital pool.  The script
 * transfers APH from the user's token account to the program's capital
 * pool PDA and records the staked amount on chain.  Update the APH
 * mint and policy addresses before running.
 */

import * as anchor from '@coral-xyz/anchor';
import { Program, Wallet, web3 } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import idl from '../target/idl/apollo_core.json';
import { PROGRAM_ID, getConfigPda, getCapitalPoolPda } from './constants';

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = new Program(idl as anchor.Idl, PROGRAM_ID, provider);
  const wallet = provider.wallet as Wallet;
  const user = wallet.payer as web3.Keypair;

  // Replace with the APH mint address deployed for your environment
  const aphMint = new PublicKey('ReplaceWithAPhMint');
  const amount = 1_000_000; // amount of APH to stake (in smallest unit)

  const [configPda] = getConfigPda();
  const [capitalPoolPda] = getCapitalPoolPda();
  const [stakePda] = PublicKey.findProgramAddressSync(
    [Buffer.from('stake'), user.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const userAph = await getAssociatedTokenAddress(aphMint, user.publicKey);

  const txSig = await program.methods
    .stakeAph(new anchor.BN(amount))
    .accounts({
      config: configPda,
      stake: stakePda,
      authority: user.publicKey,
      userAphAccount: userAph,
      capitalPool: capitalPoolPda,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([])
    .rpc();
  console.log('Stake transaction:', txSig);
}

main().catch((err) => {
  console.error(err);
});

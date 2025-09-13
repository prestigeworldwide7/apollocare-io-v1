/*
 * This is a skeleton test suite for the ApolloCare v1 program.  It uses
 * Mocha and the Anchor testing framework.  To run these tests locally,
 * install the Anchor CLI and run `anchor test`.  These tests are
 * illustrative and may require modification to fit your local setup.
 */

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import { expect } from 'chai';

describe('apollo_core', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Program and accounts will be initialised in the tests.
  let program: Program;
  let config: PublicKey;
  let authority: anchor.web3.Keypair;
  let policy: PublicKey;

  before(async () => {
    // Load IDL and program ID after building the program.
    const idl = await anchor.Program.fetchIdl('apollo_core', provider);
    const programId = new PublicKey('Apoll1CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCcApH');
    program = new anchor.Program(idl!, programId, provider);
  });

  it('Initializes the protocol', async () => {
    authority = anchor.web3.Keypair.generate();
    const usdcMint = anchor.web3.Keypair.generate().publicKey; // placeholder
    const aphMint = anchor.web3.Keypair.generate().publicKey; // placeholder
    const [configPda] = await PublicKey.findProgramAddress(
      [Buffer.from('config')],
      program.programId,
    );
    await program.methods
      .initialize(usdcMint, aphMint, new anchor.BN(500_000))
      .accounts({
        config: configPda,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc();
    config = configPda;
    const cfg = await program.account.config.fetch(config);
    expect(cfg.authority.equals(authority.publicKey)).to.be.true;
  });
});

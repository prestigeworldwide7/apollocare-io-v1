# ApolloCare.io v1


ApolloCare is a decentralized, community‑owned health coverage platform built on the high throughput Solana blockchain.  The vision is to replace today’s opaque and inefficient U.S. insurance system with transparent on‑chain programs, a member‑controlled DAO and fair market pricing.  This repository contains version 1 of the core protocol contracts and a thin client for interacting with them.  The code is designed for educational and research purposes and should be audited before production use.

## Motivation

The Apollo protocol addresses three structural problems in U.S. healthcare: administrative overhead, opaque pricing and misaligned incentives.  The whitepaper notes that administrative complexity accounts for roughly **30 %** of excess costs and is the largest opportunity for savings【295456363413057†L530-L557】.  Traditional insurers impose high premiums while only paying out 60–80 % of premiums in claims; Apollo targets a medical‑loss ratio greater than 90 %.  By automating membership, claims and treasury management as smart contracts on Solana, the protocol eliminates manual bureaucracy and reduces fees to near‑zero, while the DAO ensures that policy rules and premiums are set transparently by members【295456363413057†L530-L557】.

## Architectural overview

The architecture is hybrid: sensitive patient information is never stored on‑chain.  Smart contracts on Solana implement the membership registry, premium pool, claims state machine and governance modules【295456363413057†L558-L579】.  Off‑chain infrastructure stores encrypted claim documents in HIPAA‑compliant storage and records their cryptographic hashes on chain【295456363413057†L586-L619】.  A set of oracles (not yet implemented in v1) will provide price feeds, provider verification and handle fiat payout bridges【295456363413057†L621-L650】.  The figure below summarizes the key on‑chain components:

- **Policy & Member Registry** – tracks coverage plans and member enrolments.  Members are issued an NFT that serves as proof of coverage.
- **Premium & Capital Pool** – holds the USDC premium pool and staked $APH tokens.  Contributors pay monthly premiums into the pool, while stakers provide a capital backstop and earn protocol rewards【295456363413057†L558-L579】.
- **Claims Contract** – a state machine for submitting, adjudicating and paying claims.  Routine claims below a threshold are automatically approved and paid, while larger claims are sent to a decentralized Claims Committee for expert review【295456363413057†L700-L748】.
- **Governance Modules** – facilitate proposal creation, voting and execution.  The DAO controls parameters such as premium levels, coverage definitions and contract upgrades【295456363413057†L1014-L1085】.

## Token economy ($APH)

ApolloCare uses a native Solana token, **$APH**, to align incentives across members, reviewers and capital providers.  $APH has a fixed maximum supply of **1 billion** tokens.  Initial distribution allocates 50 % to the community and ecosystem fund, 20 % to the core team, 15 % to seed and strategic investors and 15 % to the insurance reserve【295456363413057†L936-L960】.  Token holders participate in governance, provide underwriting capital through staking, and earn rewards for activities such as claims review and community growth【295456363413057†L817-L872】.

Members who stake $APH receive discounts on their monthly protocol fee.  The tokenomics paper specifies five tiers – Bronze, Silver, Gold, Platinum and Enterprise – with increasing required stakes (from 1,000 APH up to 250,000 APH) and rebates ranging from 5 % to 40 %【939762332769470†L154-L218】.  Time‑Weighted Average Balances (TWAB) determine tier eligibility to prevent users from briefly staking to game the system【939762332769470†L217-L229】.  These rebates apply only to the platform’s administrative fee, not the core risk premium, preserving regulatory compliance【939762332769470†L229-L246】.

Claims are adjudicated by a randomly selected panel of five stakers.  Reviewers must post a bond proportional to the claim amount; the bond is discounted for high‑reputation reviewers and increased for low‑reputation reviewers【939762332769470†L316-L337】.  Reviewers who act dishonestly can be slashed, while honest reviewers earn $APH rewards and a share of protocol fees【939762332769470†L290-L314】.

## Contents of this repository

This repository contains a reference implementation of the ApolloCare protocol for Solana.  It is divided into several components:

| Path | Description |
| --- | --- |
| `Anchor.toml` | Anchor configuration specifying program locations and cluster settings. |
| `programs/apollo_core/` | The on‑chain smart contract written in Rust using the Anchor framework.  It implements policies, member enrolment, premium payments, staking, claims submission and simple claim approval logic. |
| `client/` | A TypeScript client that demonstrates how to interact with the on‑chain program using `@coral-xyz/anchor` and `@solana/web3.js`.  Scripts are provided for enrollment, premium payment and claim submission. |
| `tests/` | Example Anchor tests (WIP) to validate the basic functionality of the program. |
| `docs/` | Supplementary documents and references extracted from the whitepaper and tokenomics design. |

### Limitations of v1

This version implements a minimal viable protocol for demonstration purposes.  Several features from the whitepaper and tokenomics design remain to be implemented in future versions:

- **Governance:**  A full DAO module for proposal creation, voting and execution is not included.  Policy creation and parameter updates are controlled by a designated authority.  Future versions should integrate SPL Governance or a custom DAO.
- **TWAB and discount tiers:**  The staking module simply records stake amounts; it does not compute time‑weighted averages or enforce tier rules.  Discounts are not automatically applied to fees in this version.
- **Claims committee and reviewer selection:**  All claims above the fast‑lane threshold must be manually approved by the authority account.  Decentralized reviewer selection, bonding, slashing and reward distribution are placeholders.
- **Off‑chain integrations:**  The dApp provided in `client/` is minimal and does not encrypt claim documents or interact with oracles.  A production deployment should integrate HIPAA‑compliant storage and price oracles.
- 

## Getting started

The code is structured to be compiled with [Anchor](https://www.anchor-lang.com) and deployed to a local Solana cluster.  The following steps outline a typical development workflow:

1. **Install dependencies** – Install [Rust](https://www.rust-lang.org/tools/install), [Solana CLI tools](https://docs.solana.com/cli/install-solana-cli-tools) and [Anchor](https://docs.anchor-lang.com/installation).  Then install Node.js packages in the `client/` directory:

   ```bash
   # Install packages for the client scripts
   cd client
   npm install
   ```

2. **Start a local cluster** – Launch a local Solana test validator in one terminal:

   ```bash
   solana-test-validator -l ledger --reset
   ```

3. **Deploy the program** – In another terminal, build and deploy the `apollo_core` program:

   ```bash
   anchor build
   anchor deploy
   ```

   The deployment outputs program addresses that must be updated in `client/src/constants.ts`.

4. **Run the client scripts** – Use the provided scripts to enroll a user, pay premiums and submit claims:

   ```bash
   # from the repository root
   ts-node client/src/enroll.ts
   ts-node client/src/pay_premium.ts
   ts-node client/src/submit_claim.ts
   ```

Refer to the comments in each script for details on usage and configuration.

## Contributing

Contributions are welcome!  Please review the design documents and understand the regulatory and ethical considerations before submitting pull requests.  Future improvements could include a full DAO implementation, TWAB staking logic, decentralized claims review and integration with USDC onramps/off‑ramps.  Open issues describe additional tasks.

## References

For a complete description of the ApolloCare vision and economic model, please refer to the official whitepaper and tokenomics design.  Key concepts referenced in this repository include the on‑chain component definitions【295456363413057†L558-L579】, the hybrid off‑chain architecture【295456363413057†L586-L619】, the member journey and claim workflow【295456363413057†L700-L748】, the $APH token functions【295456363413057†L817-L872】 and the tiered discount schedule【939762332769470†L154-L218】.


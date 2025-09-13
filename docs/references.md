# ApolloCare References

This file contains selected excerpts from the official whitepaper and tokenomics documents.  They are provided here for convenient reference when working on the codebase.  Each excerpt includes a citation back to the source PDF.

## Platform architecture

The Apollo protocol is built on the Solana blockchain because of its high throughput (65,000\u00a0+ transactions per second) and low fees (under $0.01 per transaction)【295456363413057†L530-L557】.  On‑chain smart contracts implement a policy and member registry, premium and capital pools, a claims state machine and governance modules【295456363413057†L558-L579】.  Sensitive data are stored off chain; only hashes are recorded on chain to ensure tamper evidence【295456363413057†L586-L619】.

## Member journey and claims workflow

Members enroll via a digital onboarding flow that includes KYC/AML verification and plan selection.  Upon paying the first premium, the smart contract marks them as active and issues a proof of coverage NFT (not implemented in v1)【295456363413057†L677-L698】.  Claims are submitted through the app by entering basic information and uploading an itemized receipt; the data are encrypted off chain and only a hash is sent on chain【295456363413057†L700-L720】.  Claims below a threshold (e.g. \$500) are processed instantly by the smart contract with real‑time reimbursement【295456363413057†L726-L744】.  Larger or complex claims are reviewed by an elected Claims Committee, whose members are compensated in $APH and must stake a performance bond【295456363413057†L753-L778】.

## Token economics and staking

The APH token serves as the utility and governance token for the protocol.  It has a fixed supply of 1\u00a0billion tokens with initial allocations to the community fund, core team, investors and an insurance reserve【295456363413057†L936-L960】.  Token holders may stake APH into the capital pool to underwrite claims and earn rewards; staking also enables participation in governance and, in the future, discounts on protocol fees【295456363413057†L817-L872】.  Five premium discount tiers – Bronze (1,000\u00a0APH), Silver (5,000\u00a0APH), Gold (25,000\u00a0APH), Platinum (100,000\u00a0APH) and Enterprise (250,000\u00a0APH) – provide increasing rebates on the platform fee【939762332769470†L154-L218】.  Time‑weighted average balances are used to determine eligibility and prevent short‑term stake spikes【939762332769470†L217-L229】.

## Decentralized claims adjudication

For high‑value claims that exceed the fast‑lane threshold, the protocol randomly selects a panel of five reviewers from the pool of stakers to evaluate the claim【939762332769470†L290-L314】.  Reviewers must post a bond in APH proportional to the claim amount, with discounts for high‑reputation reviewers and penalties for low‑reputation reviewers【939762332769470†L316-L337】.  The panel votes to approve or deny the claim; reviewers are rewarded in APH and protocol fees for honest behaviour, while dishonest or negligent reviewers may have their bond slashed【939762332769470†L316-L337】.

## Governance

All protocol parameters, including premium levels, coverage definitions and contract upgrades, are controlled by the Apollo DAO.  Token holders propose and vote on changes, and specialized committees handle claims, risk management and treasury operations【295456363413057†L1014-L1085】.  v1 of this repository does not include a governance module; a future version should integrate with SPL\u00a0Governance or implement a custom DAO.

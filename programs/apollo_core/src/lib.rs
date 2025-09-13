use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

/// ApolloCare v1 core program.
///
/// This program implements a minimal subset of the architecture described in the
/// ApolloCare whitepaper.  It allows an authority to configure the protocol,
/// create insurance policies, enroll members, accept monthly premiums, accept
/// staking of the APH token and submit/approve claims.  It does **not**
/// implement full DAO governance, TWAB accounting, decentralized claims
/// adjudication or oracles; those features are left to future versions.

declare_id!("Apoll1CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCcApH");

#[program]
pub mod apollo_core {
    use super::*;

    /// Initializes the protocol configuration and optionally creates the
    /// program‑owned premium and capital pools.  The signer of this transaction
    /// becomes the protocol authority.  The USDC and APH mint addresses are
    /// stored for later validation.  The fast claim threshold controls the
    /// maximum claim amount that is automatically approved without manual
    /// intervention.  All amounts are expressed in the smallest unit of the
    /// respective token (e.g. USDC has 6 decimals).
    pub fn initialize(
        ctx: Context<Initialize>,
        usdc_mint: Pubkey,
        aph_mint: Pubkey,
        fast_claim_threshold: u64,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.authority = ctx.accounts.authority.key();
        cfg.usdc_mint = usdc_mint;
        cfg.aph_mint = aph_mint;
        cfg.fast_claim_threshold = fast_claim_threshold;
        cfg.next_policy_id = 0;
        cfg.bump = *ctx.bumps.get("config").unwrap();
        Ok(())
    }

    /// Creates a new insurance policy.  Only the protocol authority may call
    /// this instruction.  A policy defines a monthly premium (in USDC) and a
    /// coverage limit (maximum claim amount per claim).  Policies are stored
    /// as separate accounts so that they can be upgraded or deactivated
    /// individually.
    pub fn create_policy(
        ctx: Context<CreatePolicy>,
        monthly_premium: u64,
        coverage_limit: u64,
    ) -> Result<()> {
        require!(monthly_premium > 0, ApolloError::InvalidParameter);
        require!(coverage_limit > 0, ApolloError::InvalidParameter);
        // Only the authority can create policies.
        require_keys_eq!(ctx.accounts.authority.key(), ctx.accounts.config.authority, ApolloError::Unauthorized);
        let policy = &mut ctx.accounts.policy;
        policy.creator = ctx.accounts.authority.key();
        policy.monthly_premium = monthly_premium;
        policy.coverage_limit = coverage_limit;
        policy.bump = *ctx.bumps.get("policy").unwrap();
        // Increment the next policy id counter in the config.
        let cfg = &mut ctx.accounts.config;
        cfg.next_policy_id = cfg.next_policy_id.checked_add(1).unwrap();
        Ok(())
    }

    /// Enrolls a member into a policy.  The member pays the first monthly
    /// premium in USDC, which is transferred into the premium pool.  A
    /// Member account is created to track the user’s policy and status.  This
    /// instruction requires the user to have a USDC token account from which
    /// the premium will be debited.  The protocol does not mint a membership
    /// NFT in v1; instead the membership is tracked in the on‑chain Member
    /// account.
    pub fn enroll_member(ctx: Context<EnrollMember>) -> Result<()> {
        let cfg = &ctx.accounts.config;
        let policy = &ctx.accounts.policy;
        // Transfer the premium from the user to the premium pool.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.premium_pool.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), policy.monthly_premium)?;

        // Initialize the member.
        let member = &mut ctx.accounts.member;
        member.authority = ctx.accounts.authority.key();
        member.policy = ctx.accounts.policy.key();
        member.active = true;
        member.join_timestamp = Clock::get()?.unix_timestamp;
        member.claim_count = 0;
        member.bump = *ctx.bumps.get("member").unwrap();
        Ok(())
    }

    /// Pays an additional monthly premium for an existing member.  This
    /// instruction does not create a member; it merely transfers USDC from
    /// the user’s account into the premium pool.  It may be used to keep
    /// coverage active.  Future versions should enforce payment schedules and
    /// premium due dates.
    pub fn pay_premium(ctx: Context<PayPremium>) -> Result<()> {
        let policy = &ctx.accounts.policy;
        // Transfer premium from user to premium pool.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc_account.to_account_info(),
            to: ctx.accounts.premium_pool.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), policy.monthly_premium)?;
        Ok(())
    }

    /// Stakes APH tokens into the capital pool.  This provides additional
    /// underwriting capital for claims and, in future versions, qualifies the
    /// staker for discounts and governance power.  The staked amount is
    /// recorded in a Stake account associated with the user.  The protocol
    /// authority is not involved in staking.
    pub fn stake_aph(ctx: Context<StakeAPH>, amount: u64) -> Result<()> {
        require!(amount > 0, ApolloError::InvalidParameter);
        // Transfer APH from user to the capital pool.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_aph_account.to_account_info(),
            to: ctx.accounts.capital_pool.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;
        // Record stake.
        let stake = &mut ctx.accounts.stake;
        if stake.amount == 0 {
            // New stake.
            stake.authority = ctx.accounts.authority.key();
        } else {
            // Ensure the stake account belongs to the same authority.
            require_keys_eq!(stake.authority, ctx.accounts.authority.key(), ApolloError::Unauthorized);
        }
        stake.amount = stake.amount.checked_add(amount).unwrap();
        stake.start_timestamp = Clock::get()?.unix_timestamp;
        stake.bump = *ctx.bumps.get("stake").unwrap();
        Ok(())
    }

    /// Unstakes APH tokens and returns them to the user.  This instruction
    /// transfers the full staked amount back to the user and resets the
    /// stake account.  It does not enforce a lockup period in v1.
    pub fn unstake_aph(ctx: Context<UnstakeAPH>) -> Result<()> {
        let amount = ctx.accounts.stake.amount;
        require!(amount > 0, ApolloError::InvalidParameter);
        // Transfer APH from capital pool to user using config as signer (the
        // authority of the capital pool token account).  The seed bump must
        // correspond to the config PDA.
        let cfg = &ctx.accounts.config;
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[cfg.bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.capital_pool.to_account_info(),
            to: ctx.accounts.user_aph_account.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds), amount)?;
        // Reset stake.
        ctx.accounts.stake.amount = 0;
        Ok(())
    }

    /// Submits a claim for reimbursement.  The claim amount (in USDC) and a
    /// cryptographic hash of the off‑chain documentation are recorded.  If the
    /// amount is below the fast claim threshold specified in the config, the
    /// claim is automatically approved and paid out from the premium pool to
    /// the user’s USDC account.  Otherwise the claim is recorded with
    /// `NeedsReview` status and must be manually approved by the authority.
    pub fn submit_claim(
        ctx: Context<SubmitClaim>,
        amount: u64,
        offchain_hash: [u8; 32],
    ) -> Result<()> {
        require!(amount > 0, ApolloError::InvalidParameter);
        // Record claim.
        let claim = &mut ctx.accounts.claim;
        claim.member = ctx.accounts.member.key();
        claim.amount = amount;
        claim.hash = offchain_hash;
        claim.submitted_at = Clock::get()?.unix_timestamp;
        claim.updated_at = claim.submitted_at;
        claim.bump = *ctx.bumps.get("claim").unwrap();

        let cfg = &ctx.accounts.config;
        // Determine whether claim is small enough for automatic approval.
        if amount <= cfg.fast_claim_threshold {
            // Perform auto‑approval and payment if funds are available.
            let pool_balance = ctx.accounts.premium_pool.amount;
            require!(pool_balance >= amount, ApolloError::InsufficientPoolBalance);
            // Transfer USDC from premium pool to user's account using config as signer.
            let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[cfg.bump]]];
            let cpi_accounts = Transfer {
                from: ctx.accounts.premium_pool.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            token::transfer(CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds), amount)?;
            claim.status = ClaimStatus::Paid;
        } else {
            claim.status = ClaimStatus::NeedsReview;
        }
        // Increment member's claim count.
        let member = &mut ctx.accounts.member;
        member.claim_count = member.claim_count.checked_add(1).unwrap();
        Ok(())
    }

    /// Approves a pending claim and pays it out.  This instruction may only
    /// be called by the protocol authority.  It is intended for claims whose
    /// amount exceeds the fast claim threshold.  After approval, USDC is
    /// transferred from the premium pool to the claimant’s account.
    pub fn approve_claim(ctx: Context<ApproveClaim>) -> Result<()> {
        // Ensure caller is the authority.
        require_keys_eq!(ctx.accounts.authority.key(), ctx.accounts.config.authority, ApolloError::Unauthorized);
        let claim = &mut ctx.accounts.claim;
        // Only allow approving claims that are pending review.
        require!(claim.status == ClaimStatus::NeedsReview, ApolloError::InvalidClaimStatus);
        // Pay the claim.
        let amount = claim.amount;
        let pool_balance = ctx.accounts.premium_pool.amount;
        require!(pool_balance >= amount, ApolloError::InsufficientPoolBalance);
        let cfg = &ctx.accounts.config;
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[cfg.bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.premium_pool.to_account_info(),
            to: ctx.accounts.user_usdc_account.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds), amount)?;
        claim.status = ClaimStatus::Paid;
        claim.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Denies a pending claim.  Only the protocol authority may call this.
    pub fn deny_claim(ctx: Context<DenyClaim>) -> Result<()> {
        require_keys_eq!(ctx.accounts.authority.key(), ctx.accounts.config.authority, ApolloError::Unauthorized);
        let claim = &mut ctx.accounts.claim;
        require!(claim.status == ClaimStatus::NeedsReview, ApolloError::InvalidClaimStatus);
        claim.status = ClaimStatus::Denied;
        claim.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

/*
 * Context definitions
 */

/// Context for the `initialize` instruction.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [b"config"],
        bump,
        payer = authority,
        space = Config::LEN,
    )]
    pub config: Account<'info, Config>,
    /// The account that will become the protocol authority.  Pays for the
    /// initialization of the config account and token pools.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// Context for creating a policy.  Only the protocol authority may call
/// this instruction.
#[derive(Accounts)]
pub struct CreatePolicy<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = authority,
        space = Policy::LEN,
        seeds = [b"policy", &config.next_policy_id.to_le_bytes()],
        bump
    )]
    pub policy: Account<'info, Policy>,
    /// The authority must sign and pay for the account creation.
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Context for enrolling a member.  The user pays the first premium in USDC.
#[derive(Accounts)]
pub struct EnrollMember<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(has_one = creator)]
    pub policy: Account<'info, Policy>,
    #[account(
        init,
        payer = authority,
        space = Member::LEN,
        seeds = [b"member", authority.key().as_ref()],
        bump
    )]
    pub member: Account<'info, Member>,
    /// The user joining the policy.  Must provide USDC to pay the first premium.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// User's USDC token account to debit premium from.
    #[account(mut, constraint = user_usdc_account.mint == config.usdc_mint)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    /// Program's premium pool token account that collects USDC premiums.  Must
    /// be owned by the config PDA.
    #[account(mut, constraint = premium_pool.mint == config.usdc_mint, constraint = premium_pool.owner == config.key())]
    pub premium_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Context for paying an additional monthly premium.
#[derive(Accounts)]
pub struct PayPremium<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub policy: Account<'info, Policy>,
    /// Member account is not needed here; premium payments are open.
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, constraint = user_usdc_account.mint == config.usdc_mint)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = premium_pool.mint == config.usdc_mint, constraint = premium_pool.owner == config.key())]
    pub premium_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Context for staking APH into the capital pool.
#[derive(Accounts)]
pub struct StakeAPH<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        init_if_needed,
        payer = authority,
        space = Stake::LEN,
        seeds = [b"stake", authority.key().as_ref()],
        bump
    )]
    pub stake: Account<'info, Stake>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, constraint = user_aph_account.mint == config.aph_mint)]
    pub user_aph_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = capital_pool.mint == config.aph_mint, constraint = capital_pool.owner == config.key())]
    pub capital_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Context for unstaking APH.
#[derive(Accounts)]
pub struct UnstakeAPH<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut, seeds = [b"stake", authority.key().as_ref()], bump = stake.bump)]
    pub stake: Account<'info, Stake>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, constraint = user_aph_account.mint == config.aph_mint)]
    pub user_aph_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = capital_pool.mint == config.aph_mint, constraint = capital_pool.owner == config.key())]
    pub capital_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Context for submitting a claim.
#[derive(Accounts)]
pub struct SubmitClaim<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut, seeds = [b"member", authority.key().as_ref()], bump = member.bump)]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// The policy on which the claim is being made.  Included for future
    /// validations.
    pub policy: Account<'info, Policy>,
    #[account(
        init,
        payer = authority,
        space = Claim::LEN,
        seeds = [b"claim", member.key().as_ref(), &member.claim_count.to_le_bytes()],
        bump
    )]
    pub claim: Account<'info, Claim>,
    #[account(mut, constraint = premium_pool.mint == config.usdc_mint, constraint = premium_pool.owner == config.key())]
    pub premium_pool: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_usdc_account.mint == config.usdc_mint)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Context for approving a claim.
#[derive(Accounts)]
pub struct ApproveClaim<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut, seeds = [b"member", member.authority.as_ref()], bump = member.bump)]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub claim: Account<'info, Claim>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, constraint = premium_pool.mint == config.usdc_mint, constraint = premium_pool.owner == config.key())]
    pub premium_pool: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_usdc_account.mint == config.usdc_mint)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Context for denying a claim.
#[derive(Accounts)]
pub struct DenyClaim<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub claim: Account<'info, Claim>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/*
 * Account types
 */

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub aph_mint: Pubkey,
    pub fast_claim_threshold: u64,
    pub next_policy_id: u64,
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 1;
}

#[account]
pub struct Policy {
    pub creator: Pubkey,
    pub monthly_premium: u64,
    pub coverage_limit: u64,
    pub bump: u8,
}

impl Policy {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

#[account]
pub struct Member {
    pub authority: Pubkey,
    pub policy: Pubkey,
    pub active: bool,
    pub join_timestamp: i64,
    pub claim_count: u64,
    pub bump: u8,
}

impl Member {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 8 + 1;
}

#[account]
pub struct Claim {
    pub member: Pubkey,
    pub amount: u64,
    pub status: ClaimStatus,
    pub submitted_at: i64,
    pub updated_at: i64,
    pub hash: [u8; 32],
    pub bump: u8,
}

impl Claim {
    pub const LEN: usize = 8 + 32 + 8 + 1 + 8 + 8 + 32 + 1;
}

#[account]
pub struct Stake {
    pub authority: Pubkey,
    pub amount: u64,
    pub start_timestamp: i64,
    pub bump: u8,
}

impl Stake {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

/// Claim status enumeration.  The number of variants is small and fits in a
/// single byte when serialized.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ClaimStatus {
    /// Claim was submitted and awaits processing.  In v1 this state is not
    /// used because claims are either auto‑paid or marked for review.
    Submitted,
    /// Claim is automatically approved and has been paid.
    Paid,
    /// Claim requires manual review and approval by the authority.
    NeedsReview,
    /// Claim was manually denied.
    Denied,
}

/*
 * Custom error codes
 */

#[error_code]
pub enum ApolloError {
    #[msg("Unauthorized: caller does not have permission to perform this action")]
    Unauthorized,
    #[msg("Invalid parameter supplied to instruction")]
    InvalidParameter,
    #[msg("Insufficient funds in the premium pool to pay the claim")]
    InsufficientPoolBalance,
    #[msg("Invalid claim status for this operation")]
    InvalidClaimStatus,
}

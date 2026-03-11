use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use crate::state::{PlatformConfig, JobAccount, JobStatus};
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct CreateJob<'info> {
    #[account(mut)]
    pub poster: Signer<'info>,

    #[account(
        mut,
        seeds = [b"platform"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(
        init,
        payer = poster,
        space = 8 + JobAccount::INIT_SPACE,
        seeds = [b"job", poster.key().as_ref(), platform_config.job_counter.to_le_bytes().as_ref()],
        bump,
    )]
    pub job_account: Account<'info, JobAccount>,

    #[account(
        init,
        payer = poster,
        token::mint = usdc_mint,
        token::authority = escrow_authority,
        seeds = [b"escrow", job_account.key().as_ref()],
        bump,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority over escrow token accounts
    #[account(
        seeds = [b"escrow_authority"],
        bump,
    )]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = poster_token_account.owner == poster.key(),
        constraint = poster_token_account.mint == usdc_mint.key(),
    )]
    pub poster_token_account: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateJob>,
    amount: u64,
    deadline: Option<i64>,
) -> Result<()> {
    require!(amount > 0, ErrandError::ZeroAmount);

    let clock = Clock::get()?;

    // Transfer USDC from poster to escrow
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.poster_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.poster.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Initialize job account
    let config = &mut ctx.accounts.platform_config;
    let job = &mut ctx.accounts.job_account;

    job.poster = ctx.accounts.poster.key();
    job.agent = None;
    job.escrow_token_account = ctx.accounts.escrow_token_account.key();
    job.amount = amount;
    job.status = JobStatus::Open;
    job.result_hash = None;
    job.rating = None;
    job.created_at = clock.unix_timestamp;
    job.deadline = deadline;
    job.job_id = config.job_counter;
    job.bump = ctx.bumps.job_account;

    // Increment global job counter
    config.job_counter = config.job_counter.checked_add(1).ok_or(ErrandError::Overflow)?;

    msg!("Job {} created: amount={}", job.job_id, amount);
    Ok(())
}

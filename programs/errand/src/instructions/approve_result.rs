use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{PlatformConfig, JobAccount, AgentAccount, JobStatus};
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct ApproveResult<'info> {
    pub poster: Signer<'info>,

    #[account(
        seeds = [b"platform"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(
        mut,
        constraint = job_account.poster == poster.key() @ ErrandError::Unauthorized,
        constraint = job_account.status == JobStatus::Review @ ErrandError::InvalidJobStatus,
    )]
    pub job_account: Account<'info, JobAccount>,

    #[account(
        mut,
        seeds = [b"agent", agent_account.authority.as_ref()],
        bump = agent_account.bump,
        constraint = Some(agent_account.authority) == job_account.agent @ ErrandError::AgentNotAssigned,
    )]
    pub agent_account: Account<'info, AgentAccount>,

    #[account(
        mut,
        constraint = escrow_token_account.key() == job_account.escrow_token_account @ ErrandError::Unauthorized,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority over escrow token accounts
    #[account(
        seeds = [b"escrow_authority"],
        bump,
    )]
    pub escrow_authority: UncheckedAccount<'info>,

    /// Agent's USDC token account to receive payment
    #[account(
        mut,
        constraint = agent_token_account.owner == agent_account.authority,
    )]
    pub agent_token_account: Account<'info, TokenAccount>,

    /// Platform treasury token account for fee collection
    #[account(
        mut,
        constraint = treasury_token_account.owner == platform_config.treasury,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ApproveResult>, rating: u8) -> Result<()> {
    require!(rating >= 1 && rating <= 5, ErrandError::InvalidRating);

    let job = &ctx.accounts.job_account;
    let fee_bps = ctx.accounts.platform_config.fee_bps as u64;

    // Calculate fee and agent payment
    let fee_amount = job
        .amount
        .checked_mul(fee_bps)
        .ok_or(ErrandError::Overflow)?
        .checked_div(10_000)
        .ok_or(ErrandError::Overflow)?;
    let agent_amount = job
        .amount
        .checked_sub(fee_amount)
        .ok_or(ErrandError::Overflow)?;

    // PDA signer seeds for escrow authority
    let escrow_authority_bump = ctx.bumps.escrow_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[b"escrow_authority", &[escrow_authority_bump]]];

    // Transfer agent payment from escrow
    let transfer_to_agent = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.agent_token_account.to_account_info(),
            authority: ctx.accounts.escrow_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_to_agent, agent_amount)?;

    // Transfer fee to treasury
    let transfer_to_treasury = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.treasury_token_account.to_account_info(),
            authority: ctx.accounts.escrow_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_to_treasury, fee_amount)?;

    // Update job
    let job = &mut ctx.accounts.job_account;
    job.status = JobStatus::Complete;
    job.rating = Some(rating);

    // Update agent reputation
    let agent = &mut ctx.accounts.agent_account;
    agent.jobs_completed = agent
        .jobs_completed
        .checked_add(1)
        .ok_or(ErrandError::Overflow)?;
    agent.total_rating_sum = agent
        .total_rating_sum
        .checked_add(rating as u32)
        .ok_or(ErrandError::Overflow)?;

    msg!(
        "Job {} approved: agent paid {}, fee {}, rating {}",
        job.job_id,
        agent_amount,
        fee_amount,
        rating
    );
    Ok(())
}

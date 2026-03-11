use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{JobAccount, JobStatus};
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct CancelJob<'info> {
    pub poster: Signer<'info>,

    #[account(
        mut,
        constraint = job_account.poster == poster.key() @ ErrandError::Unauthorized,
        constraint = job_account.status == JobStatus::Open @ ErrandError::InvalidJobStatus,
    )]
    pub job_account: Account<'info, JobAccount>,

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

    /// Poster's token account to receive the refund
    #[account(
        mut,
        constraint = poster_token_account.owner == poster.key(),
    )]
    pub poster_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelJob>) -> Result<()> {
    let job = &ctx.accounts.job_account;
    let refund_amount = job.amount;

    // PDA signer seeds for escrow authority
    let escrow_authority_bump = ctx.bumps.escrow_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[b"escrow_authority", &[escrow_authority_bump]]];

    // Transfer full amount back to poster
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.poster_token_account.to_account_info(),
            authority: ctx.accounts.escrow_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, refund_amount)?;

    // Update job status
    let job = &mut ctx.accounts.job_account;
    job.status = JobStatus::Cancelled;

    msg!("Job {} cancelled, {} refunded", job.job_id, refund_amount);
    Ok(())
}

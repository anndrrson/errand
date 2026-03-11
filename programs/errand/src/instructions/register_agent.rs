use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{PlatformConfig, AgentAccount};
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"platform"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + AgentAccount::INIT_SPACE,
        seeds = [b"agent", authority.key().as_ref()],
        bump,
    )]
    pub agent_account: Account<'info, AgentAccount>,

    /// Agent's USDC token account to stake from
    #[account(
        mut,
        constraint = agent_token_account.owner == authority.key(),
    )]
    pub agent_token_account: Account<'info, TokenAccount>,

    /// Staking vault PDA token account
    #[account(
        mut,
        seeds = [b"stake_vault"],
        bump,
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterAgent>, stake_amount: u64) -> Result<()> {
    let min_stake = ctx.accounts.platform_config.min_stake;
    require!(stake_amount >= min_stake, ErrandError::InsufficientStake);

    let clock = Clock::get()?;

    // Transfer stake from agent to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.agent_token_account.to_account_info(),
            to: ctx.accounts.stake_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, stake_amount)?;

    // Initialize agent account
    let agent = &mut ctx.accounts.agent_account;
    agent.authority = ctx.accounts.authority.key();
    agent.stake_amount = stake_amount;
    agent.jobs_completed = 0;
    agent.total_rating_sum = 0;
    agent.dispute_count = 0;
    agent.active = true;
    agent.created_at = clock.unix_timestamp;
    agent.bump = ctx.bumps.agent_account;

    msg!("Agent registered: stake={}", stake_amount);
    Ok(())
}

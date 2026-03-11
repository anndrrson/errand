use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::AgentAccount;
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct DeregisterAgent<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"agent", authority.key().as_ref()],
        bump = agent_account.bump,
        constraint = agent_account.authority == authority.key() @ ErrandError::Unauthorized,
        constraint = agent_account.active @ ErrandError::AgentNotActive,
    )]
    pub agent_account: Account<'info, AgentAccount>,

    /// Agent's USDC token account to receive stake back
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

    /// CHECK: PDA authority over stake vault
    #[account(
        seeds = [b"stake_vault_authority"],
        bump,
    )]
    pub stake_vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<DeregisterAgent>) -> Result<()> {
    let agent = &ctx.accounts.agent_account;
    let refund_amount = agent.stake_amount;

    // PDA signer seeds for stake vault authority
    let vault_authority_bump = ctx.bumps.stake_vault_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[b"stake_vault_authority", &[vault_authority_bump]]];

    // Transfer stake back to agent
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.agent_token_account.to_account_info(),
            authority: ctx.accounts.stake_vault_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, refund_amount)?;

    // Deactivate agent
    let agent = &mut ctx.accounts.agent_account;
    agent.active = false;
    agent.stake_amount = 0;

    msg!("Agent deregistered, stake {} returned", refund_amount);
    Ok(())
}

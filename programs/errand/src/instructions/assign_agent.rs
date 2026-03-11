use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, JobAccount, AgentAccount, JobStatus};
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct AssignAgent<'info> {
    #[account(
        constraint = authority.key() == platform_config.authority @ ErrandError::Unauthorized,
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"platform"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(
        mut,
        constraint = job_account.status == JobStatus::Open @ ErrandError::InvalidJobStatus,
    )]
    pub job_account: Account<'info, JobAccount>,

    #[account(
        seeds = [b"agent", agent_account.authority.as_ref()],
        bump = agent_account.bump,
        constraint = agent_account.active @ ErrandError::AgentNotActive,
    )]
    pub agent_account: Account<'info, AgentAccount>,
}

pub fn handler(ctx: Context<AssignAgent>) -> Result<()> {
    let job = &mut ctx.accounts.job_account;
    let agent = &ctx.accounts.agent_account;

    job.agent = Some(agent.authority);
    job.status = JobStatus::Claimed;

    msg!("Job {} assigned to agent {}", job.job_id, agent.authority);
    Ok(())
}

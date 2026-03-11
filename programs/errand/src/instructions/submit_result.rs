use anchor_lang::prelude::*;
use crate::state::{JobAccount, JobStatus};
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct SubmitResult<'info> {
    pub agent: Signer<'info>,

    #[account(
        mut,
        constraint = job_account.agent == Some(agent.key()) @ ErrandError::AgentNotAssigned,
        constraint = (
            job_account.status == JobStatus::Claimed ||
            job_account.status == JobStatus::InProgress
        ) @ ErrandError::InvalidJobStatus,
    )]
    pub job_account: Account<'info, JobAccount>,
}

pub fn handler(ctx: Context<SubmitResult>, result_hash: [u8; 32]) -> Result<()> {
    let job = &mut ctx.accounts.job_account;

    job.result_hash = Some(result_hash);
    job.status = JobStatus::Review;

    msg!("Job {} result submitted for review", job.job_id);
    Ok(())
}

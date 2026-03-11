use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

pub use instructions::*;

declare_id!("Errand1111111111111111111111111111111111111");

#[program]
pub mod errand {
    use super::*;

    /// Initialize the platform configuration.
    pub fn initialize(
        ctx: Context<Initialize>,
        fee_bps: u16,
        min_stake: u64,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, fee_bps, min_stake)
    }

    /// Create a new job and deposit USDC into escrow.
    pub fn create_job(
        ctx: Context<CreateJob>,
        amount: u64,
        deadline: Option<i64>,
    ) -> Result<()> {
        instructions::create_job::handler(ctx, amount, deadline)
    }

    /// Platform assigns a registered agent to an open job.
    pub fn assign_agent(ctx: Context<AssignAgent>) -> Result<()> {
        instructions::assign_agent::handler(ctx)
    }

    /// Agent submits a result hash (SHA-256 of deliverable).
    pub fn submit_result(
        ctx: Context<SubmitResult>,
        result_hash: [u8; 32],
    ) -> Result<()> {
        instructions::submit_result::handler(ctx, result_hash)
    }

    /// Poster approves the result, releasing escrow to agent and fee to treasury.
    pub fn approve_result(
        ctx: Context<ApproveResult>,
        rating: u8,
    ) -> Result<()> {
        instructions::approve_result::handler(ctx, rating)
    }

    /// Poster cancels an open job and receives a full refund.
    pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
        instructions::cancel_job::handler(ctx)
    }

    /// Register as an agent by staking USDC.
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        stake_amount: u64,
    ) -> Result<()> {
        instructions::register_agent::handler(ctx, stake_amount)
    }

    /// Deregister an agent and return their stake.
    pub fn deregister_agent(ctx: Context<DeregisterAgent>) -> Result<()> {
        instructions::deregister_agent::handler(ctx)
    }
}

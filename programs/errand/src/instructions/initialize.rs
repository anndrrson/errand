use anchor_lang::prelude::*;
use crate::state::PlatformConfig;
use crate::errors::ErrandError;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + PlatformConfig::INIT_SPACE,
        seeds = [b"platform"],
        bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    /// CHECK: Treasury wallet, validated by caller
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: Dispute resolver, validated by caller
    pub dispute_resolver: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    fee_bps: u16,
    min_stake: u64,
) -> Result<()> {
    require!(fee_bps <= 10_000, ErrandError::InvalidFeeBps);

    let config = &mut ctx.accounts.platform_config;
    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.fee_bps = fee_bps;
    config.dispute_resolver = ctx.accounts.dispute_resolver.key();
    config.min_stake = min_stake;
    config.job_counter = 0;
    config.bump = ctx.bumps.platform_config;

    msg!("Platform initialized: fee={}bps, min_stake={}", fee_bps, min_stake);
    Ok(())
}

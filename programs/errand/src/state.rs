use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PlatformConfig {
    /// Authority that controls the platform
    pub authority: Pubkey,
    /// Treasury wallet for collecting fees
    pub treasury: Pubkey,
    /// Fee in basis points (e.g. 500 = 5%)
    pub fee_bps: u16,
    /// Authority that can resolve disputes
    pub dispute_resolver: Pubkey,
    /// Minimum USDC stake required for agents
    pub min_stake: u64,
    /// Monotonically increasing job counter
    pub job_counter: u64,
    /// PDA bump
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct JobAccount {
    /// The user who posted this job
    pub poster: Pubkey,
    /// The agent assigned to this job (None if open)
    pub agent: Option<Pubkey>,
    /// The escrow token account holding USDC for this job
    pub escrow_token_account: Pubkey,
    /// USDC amount deposited (in smallest unit)
    pub amount: u64,
    /// Current status of the job
    pub status: JobStatus,
    /// SHA-256 hash of the deliverable submitted by the agent
    pub result_hash: Option<[u8; 32]>,
    /// Poster's rating of the agent (1-5)
    pub rating: Option<u8>,
    /// Unix timestamp of job creation
    pub created_at: i64,
    /// Optional deadline (unix timestamp)
    pub deadline: Option<i64>,
    /// Unique job identifier
    pub job_id: u64,
    /// PDA bump
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum JobStatus {
    Open,
    Claimed,
    InProgress,
    Review,
    Complete,
    Disputed,
    Cancelled,
}

#[account]
#[derive(InitSpace)]
pub struct AgentAccount {
    /// Wallet authority of the agent
    pub authority: Pubkey,
    /// Amount of USDC staked
    pub stake_amount: u64,
    /// Total jobs completed
    pub jobs_completed: u32,
    /// Sum of all ratings received
    pub total_rating_sum: u32,
    /// Number of disputes against this agent
    pub dispute_count: u32,
    /// Whether the agent is currently active
    pub active: bool,
    /// Unix timestamp of registration
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}

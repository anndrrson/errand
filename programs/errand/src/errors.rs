use anchor_lang::prelude::*;

#[error_code]
pub enum ErrandError {
    #[msg("Job is not in the expected status")]
    InvalidJobStatus,

    #[msg("Unauthorized: signer does not have permission for this action")]
    Unauthorized,

    #[msg("Agent is not registered or not active")]
    AgentNotActive,

    #[msg("Agent has active jobs and cannot deregister")]
    AgentHasActiveJobs,

    #[msg("Insufficient stake amount")]
    InsufficientStake,

    #[msg("Rating must be between 1 and 5")]
    InvalidRating,

    #[msg("Job deadline has passed")]
    DeadlinePassed,

    #[msg("Fee basis points must be <= 10000")]
    InvalidFeeBps,

    #[msg("Amount must be greater than zero")]
    ZeroAmount,

    #[msg("Result hash is required")]
    MissingResultHash,

    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Agent is already registered")]
    AgentAlreadyRegistered,

    #[msg("Agent is not assigned to this job")]
    AgentNotAssigned,
}

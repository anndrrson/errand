use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Task kinds & categories
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskKind {
    /// Execute once and return the result.
    OneShot,
    /// Run on a cron schedule (e.g. "0 9 * * MON" = every Monday 9am).
    Recurring { cron: String },
    /// Watch for a condition, alert when met.
    Monitor {
        condition: String,
        check_interval_seconds: u64,
    },
    /// Multi-step pipeline with optional human checkpoints.
    Pipeline { steps: Vec<PipelineStep> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub description: String,
    /// If true, the pipeline pauses here and waits for human approval.
    pub requires_approval: bool,
    /// Optional dependency on a previous step index.
    pub depends_on: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    Research,
    Content,
    Data,
    Crypto,
    Monitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    WaitingApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: String,
    pub kind: TaskKind,
    pub category: TaskCategory,
    pub status: TaskStatus,
    pub webhook_url: Option<String>,
    pub email_notify: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub next_run_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Task runs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRun {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: RunStatus,
    pub steps_completed: u32,
    pub result: Option<String>,
    pub result_hash: Option<String>,
    pub cost_credits: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Agents
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub categories: Vec<TaskCategory>,
    pub model: String,
    pub tools: Vec<String>,
    pub avg_rating: f32,
    pub jobs_completed: u32,
}

// ---------------------------------------------------------------------------
// Credits
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditBalance {
    pub owner_id: Uuid,
    pub balance: i64,
    pub lifetime_earned: i64,
    pub lifetime_spent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub amount: i64,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthTokenResponse {
    pub token: String,
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub kind: TaskKind,
    pub category: TaskCategory,
    pub webhook_url: Option<String>,
    pub email_notify: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub task: Task,
    pub runs: Vec<TaskRun>,
    pub stream_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveStepRequest {
    pub task_id: Uuid,
    pub run_id: Uuid,
    pub step_index: usize,
    pub approved: bool,
    pub feedback: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentListResponse {
    pub agents: Vec<Agent>,
}

#[derive(Debug, Serialize)]
pub struct CreditBalanceResponse {
    pub balance: CreditBalance,
}

#[derive(Debug, Serialize)]
pub struct CreditHistoryResponse {
    pub transactions: Vec<CreditTransaction>,
}

#[derive(Debug, Deserialize)]
pub struct RedeemCodeRequest {
    pub code: String,
}

/// Result produced by an agent execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub result_text: String,
    pub result_hash: String,
    pub sources: Vec<String>,
}

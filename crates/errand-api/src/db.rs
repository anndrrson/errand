use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use errand_types::{
    Agent, CreditBalance, CreditTransaction, RunStatus, Task, TaskCategory, TaskKind, TaskRun,
    TaskStatus,
};

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

pub async fn create_user(pool: &PgPool, email: &str, password_hash: &str) -> Result<Uuid, AppError> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
    )
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn get_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String)>, AppError> {
    let row: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

pub async fn create_task(pool: &PgPool, task: &Task) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO tasks (id, owner_id, title, description, kind, category, status,
                           webhook_url, email_notify, next_run_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(task.id)
    .bind(task.owner_id)
    .bind(&task.title)
    .bind(&task.description)
    .bind(serde_json::to_value(&task.kind).unwrap_or_default())
    .bind(serde_json::to_string(&task.category).unwrap_or_default().trim_matches('"'))
    .bind(serde_json::to_string(&task.status).unwrap_or_default().trim_matches('"'))
    .bind(&task.webhook_url)
    .bind(&task.email_notify)
    .bind(task.next_run_at)
    .bind(task.created_at)
    .bind(task.updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_task(pool: &PgPool, id: Uuid) -> Result<Option<Task>, AppError> {
    let row = sqlx::query_as::<_, TaskRow>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(Task::from))
}

pub async fn list_tasks_for_owner(pool: &PgPool, owner_id: Uuid) -> Result<Vec<Task>, AppError> {
    let rows = sqlx::query_as::<_, TaskRow>(
        "SELECT * FROM tasks WHERE owner_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Task::from).collect())
}

pub async fn update_task_status(
    pool: &PgPool,
    id: Uuid,
    status: TaskStatus,
) -> Result<(), AppError> {
    let status_str = serde_json::to_string(&status)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();

    sqlx::query("UPDATE tasks SET status = $1, updated_at = $2 WHERE id = $3")
        .bind(&status_str)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_task_next_run(
    pool: &PgPool,
    id: Uuid,
    next_run_at: chrono::DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE tasks SET next_run_at = $1, updated_at = $2 WHERE id = $3")
        .bind(next_run_at)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Find recurring tasks that are due to run.
pub async fn get_due_recurring_tasks(pool: &PgPool) -> Result<Vec<Task>, AppError> {
    let rows = sqlx::query_as::<_, TaskRow>(
        r#"
        SELECT * FROM tasks
        WHERE status = 'running'
          AND next_run_at IS NOT NULL
          AND next_run_at <= NOW()
          AND kind::text LIKE '%recurring%'
        ORDER BY next_run_at ASC
        LIMIT 50
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Task::from).collect())
}

/// Find monitor tasks that are due for a check.
pub async fn get_due_monitor_tasks(pool: &PgPool) -> Result<Vec<Task>, AppError> {
    let rows = sqlx::query_as::<_, TaskRow>(
        r#"
        SELECT * FROM tasks
        WHERE status = 'running'
          AND next_run_at IS NOT NULL
          AND next_run_at <= NOW()
          AND kind::text LIKE '%monitor%'
        ORDER BY next_run_at ASC
        LIMIT 50
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Task::from).collect())
}

// ---------------------------------------------------------------------------
// Task Runs
// ---------------------------------------------------------------------------

pub async fn create_task_run(pool: &PgPool, task_id: Uuid) -> Result<Uuid, AppError> {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO task_runs (task_id, status, steps_completed, cost_credits, started_at)
        VALUES ($1, 'running', 0, 0, NOW())
        RETURNING id
        "#,
    )
    .bind(task_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn get_task_run(pool: &PgPool, id: Uuid) -> Result<Option<TaskRun>, AppError> {
    let row = sqlx::query_as::<_, TaskRunRow>("SELECT * FROM task_runs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(TaskRun::from))
}

pub async fn get_runs_for_task(pool: &PgPool, task_id: Uuid) -> Result<Vec<TaskRun>, AppError> {
    let rows = sqlx::query_as::<_, TaskRunRow>(
        "SELECT * FROM task_runs WHERE task_id = $1 ORDER BY started_at DESC LIMIT 50",
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(TaskRun::from).collect())
}

pub async fn update_task_run_completed(
    pool: &PgPool,
    id: Uuid,
    status: RunStatus,
    result: Option<&str>,
    result_hash: Option<&str>,
    cost_credits: u32,
    steps_completed: u32,
) -> Result<(), AppError> {
    let status_str = serde_json::to_string(&status)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();

    sqlx::query(
        r#"
        UPDATE task_runs
        SET status = $1, result = $2, result_hash = $3,
            cost_credits = $4, steps_completed = $5, completed_at = NOW()
        WHERE id = $6
        "#,
    )
    .bind(&status_str)
    .bind(result)
    .bind(result_hash)
    .bind(cost_credits as i32)
    .bind(steps_completed as i32)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_task_run_steps(
    pool: &PgPool,
    id: Uuid,
    steps_completed: u32,
) -> Result<(), AppError> {
    sqlx::query("UPDATE task_runs SET steps_completed = $1 WHERE id = $2")
        .bind(steps_completed as i32)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Credits
// ---------------------------------------------------------------------------

pub async fn init_credit_balance(pool: &PgPool, owner_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO credit_balances (owner_id, balance, lifetime_earned, lifetime_spent) VALUES ($1, 0, 0, 0)",
    )
    .bind(owner_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_credit_balance(pool: &PgPool, owner_id: Uuid) -> Result<CreditBalance, AppError> {
    let row: Option<CreditBalanceRow> = sqlx::query_as(
        "SELECT * FROM credit_balances WHERE owner_id = $1",
    )
    .bind(owner_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(CreditBalance::from).unwrap_or(CreditBalance {
        owner_id,
        balance: 0,
        lifetime_earned: 0,
        lifetime_spent: 0,
    }))
}

/// Add a credit transaction. Positive amount = earn, negative = spend.
pub async fn add_credit_transaction(
    pool: &PgPool,
    owner_id: Uuid,
    amount: i64,
    reason: &str,
) -> Result<(), AppError> {
    // Insert transaction record
    sqlx::query(
        "INSERT INTO credit_transactions (owner_id, amount, reason) VALUES ($1, $2, $3)",
    )
    .bind(owner_id)
    .bind(amount)
    .bind(reason)
    .execute(pool)
    .await?;

    // Update balance
    if amount >= 0 {
        sqlx::query(
            r#"
            UPDATE credit_balances
            SET balance = balance + $1, lifetime_earned = lifetime_earned + $1
            WHERE owner_id = $2
            "#,
        )
        .bind(amount)
        .bind(owner_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE credit_balances
            SET balance = balance + $1, lifetime_spent = lifetime_spent + $2
            WHERE owner_id = $3
            "#,
        )
        .bind(amount)
        .bind(-amount)
        .bind(owner_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_credit_history(
    pool: &PgPool,
    owner_id: Uuid,
) -> Result<Vec<CreditTransaction>, AppError> {
    let rows = sqlx::query_as::<_, CreditTransactionRow>(
        "SELECT * FROM credit_transactions WHERE owner_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(CreditTransaction::from).collect())
}

// ---------------------------------------------------------------------------
// Agents (static registry, but stored in DB for extensibility)
// ---------------------------------------------------------------------------

pub async fn list_agents(pool: &PgPool) -> Result<Vec<Agent>, AppError> {
    let rows = sqlx::query_as::<_, AgentRow>(
        "SELECT * FROM agents ORDER BY avg_rating DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Agent::from).collect())
}

pub async fn get_agent(pool: &PgPool, id: &str) -> Result<Option<Agent>, AppError> {
    let row = sqlx::query_as::<_, AgentRow>("SELECT * FROM agents WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(Agent::from))
}

// ---------------------------------------------------------------------------
// Row types (sqlx::FromRow) — conversion layer between DB and domain types
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: Uuid,
    owner_id: Uuid,
    title: String,
    description: String,
    kind: serde_json::Value,
    category: String,
    status: String,
    webhook_url: Option<String>,
    email_notify: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    next_run_at: Option<chrono::DateTime<Utc>>,
}

impl From<TaskRow> for Task {
    fn from(r: TaskRow) -> Self {
        Task {
            id: r.id,
            owner_id: r.owner_id,
            title: r.title,
            description: r.description,
            kind: serde_json::from_value(r.kind).unwrap_or(TaskKind::OneShot),
            category: serde_json::from_value(serde_json::Value::String(r.category))
                .unwrap_or(TaskCategory::Research),
            status: serde_json::from_value(serde_json::Value::String(r.status))
                .unwrap_or(TaskStatus::Pending),
            webhook_url: r.webhook_url,
            email_notify: r.email_notify,
            created_at: r.created_at,
            updated_at: r.updated_at,
            next_run_at: r.next_run_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRunRow {
    id: Uuid,
    task_id: Uuid,
    status: String,
    steps_completed: i32,
    result: Option<String>,
    result_hash: Option<String>,
    cost_credits: i32,
    started_at: chrono::DateTime<Utc>,
    completed_at: Option<chrono::DateTime<Utc>>,
}

impl From<TaskRunRow> for TaskRun {
    fn from(r: TaskRunRow) -> Self {
        TaskRun {
            id: r.id,
            task_id: r.task_id,
            status: serde_json::from_value(serde_json::Value::String(r.status))
                .unwrap_or(RunStatus::Running),
            steps_completed: r.steps_completed as u32,
            result: r.result,
            result_hash: r.result_hash,
            cost_credits: r.cost_credits as u32,
            started_at: r.started_at,
            completed_at: r.completed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CreditBalanceRow {
    owner_id: Uuid,
    balance: i64,
    lifetime_earned: i64,
    lifetime_spent: i64,
}

impl From<CreditBalanceRow> for CreditBalance {
    fn from(r: CreditBalanceRow) -> Self {
        CreditBalance {
            owner_id: r.owner_id,
            balance: r.balance,
            lifetime_earned: r.lifetime_earned,
            lifetime_spent: r.lifetime_spent,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CreditTransactionRow {
    id: Uuid,
    owner_id: Uuid,
    amount: i64,
    reason: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<CreditTransactionRow> for CreditTransaction {
    fn from(r: CreditTransactionRow) -> Self {
        CreditTransaction {
            id: r.id,
            owner_id: r.owner_id,
            amount: r.amount,
            reason: r.reason,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AgentRow {
    id: String,
    name: String,
    description: String,
    categories: serde_json::Value,
    model: String,
    tools: serde_json::Value,
    avg_rating: f32,
    jobs_completed: i32,
}

impl From<AgentRow> for Agent {
    fn from(r: AgentRow) -> Self {
        Agent {
            id: r.id,
            name: r.name,
            description: r.description,
            categories: serde_json::from_value(r.categories).unwrap_or_default(),
            model: r.model,
            tools: serde_json::from_value(r.tools).unwrap_or_default(),
            avg_rating: r.avg_rating,
            jobs_completed: r.jobs_completed as u32,
        }
    }
}

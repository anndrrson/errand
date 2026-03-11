use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    Json,
};
use chrono::Utc;
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::AppError,
    streaming::{ProgressEvent, TaskProgressStream},
    AppState,
};
use errand_types::{
    ApproveStepRequest, CreateTaskRequest, Task, TaskKind, TaskListResponse, TaskResponse,
    TaskStatus,
};

/// Cost in credits for a task kind.
fn credit_cost(kind: &TaskKind) -> i64 {
    match kind {
        TaskKind::OneShot => 1,
        TaskKind::Recurring { .. } => 2,
        TaskKind::Monitor { .. } => 1,
        TaskKind::Pipeline { steps } => steps.len().max(1) as i64,
    }
}

/// POST /api/tasks -- create a new task
pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    // Check credit balance
    let balance = db::get_credit_balance(&state.pool, auth.user_id).await?;
    let cost = credit_cost(&req.kind);
    if balance.balance < cost {
        return Err(AppError::InsufficientCredits);
    }

    let now = Utc::now();

    // Calculate next_run_at for recurring/monitor tasks
    let next_run_at = match &req.kind {
        TaskKind::Recurring { cron } => {
            // Parse cron and find next occurrence
            parse_next_cron(cron).or(Some(now))
        }
        TaskKind::Monitor {
            check_interval_seconds,
            ..
        } => Some(now + chrono::Duration::seconds(*check_interval_seconds as i64)),
        _ => None,
    };

    let task = Task {
        id: Uuid::new_v4(),
        owner_id: auth.user_id,
        title: req.title,
        description: req.description,
        kind: req.kind,
        category: req.category,
        status: TaskStatus::Running,
        webhook_url: req.webhook_url,
        email_notify: req.email_notify,
        created_at: now,
        updated_at: now,
        next_run_at,
    };

    db::create_task(&state.pool, &task).await?;

    // Create the first run
    let run_id = db::create_task_run(&state.pool, task.id).await?;

    // Deduct credits
    db::add_credit_transaction(
        &state.pool,
        auth.user_id,
        -cost,
        &format!("task_run:{}", task.id),
    )
    .await?;

    // Send to executor
    let _ = state.executor_tx.send(run_id).await;

    let runs = db::get_runs_for_task(&state.pool, task.id).await?;
    let stream_url = Some(format!("/api/tasks/{}/stream", task.id));

    Ok(Json(TaskResponse {
        task,
        runs,
        stream_url,
    }))
}

/// GET /api/tasks -- list user's tasks
pub async fn list_tasks(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<TaskListResponse>, AppError> {
    let tasks = db::list_tasks_for_owner(&state.pool, auth.user_id).await?;
    Ok(Json(TaskListResponse { tasks }))
}

/// GET /api/tasks/:id -- get task detail with runs
pub async fn get_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if task.owner_id != auth.user_id {
        return Err(AppError::Forbidden("Not your task".into()));
    }

    let runs = db::get_runs_for_task(&state.pool, task.id).await?;

    Ok(Json(TaskResponse {
        task,
        runs,
        stream_url: None,
    }))
}

/// POST /api/tasks/:id/pause -- pause a recurring/monitor task
pub async fn pause_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if task.owner_id != auth.user_id {
        return Err(AppError::Forbidden("Not your task".into()));
    }

    if task.status != TaskStatus::Running {
        return Err(AppError::BadRequest("Task is not running".into()));
    }

    db::update_task_status(&state.pool, id, TaskStatus::Paused).await?;

    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Task disappeared".into()))?;
    let runs = db::get_runs_for_task(&state.pool, id).await?;

    Ok(Json(TaskResponse {
        task,
        runs,
        stream_url: None,
    }))
}

/// POST /api/tasks/:id/resume -- resume a paused task
pub async fn resume_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if task.owner_id != auth.user_id {
        return Err(AppError::Forbidden("Not your task".into()));
    }

    if task.status != TaskStatus::Paused {
        return Err(AppError::BadRequest("Task is not paused".into()));
    }

    db::update_task_status(&state.pool, id, TaskStatus::Running).await?;

    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Task disappeared".into()))?;
    let runs = db::get_runs_for_task(&state.pool, id).await?;

    Ok(Json(TaskResponse {
        task,
        runs,
        stream_url: None,
    }))
}

/// POST /api/tasks/:id/cancel -- cancel a task
pub async fn cancel_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if task.owner_id != auth.user_id {
        return Err(AppError::Forbidden("Not your task".into()));
    }

    if task.status == TaskStatus::Completed || task.status == TaskStatus::Cancelled {
        return Err(AppError::BadRequest("Task is already finished".into()));
    }

    db::update_task_status(&state.pool, id, TaskStatus::Cancelled).await?;

    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Task disappeared".into()))?;
    let runs = db::get_runs_for_task(&state.pool, id).await?;

    Ok(Json(TaskResponse {
        task,
        runs,
        stream_url: None,
    }))
}

/// POST /api/tasks/:id/approve -- approve a pipeline step
pub async fn approve_step(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveStepRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if task.owner_id != auth.user_id {
        return Err(AppError::Forbidden("Not your task".into()));
    }

    if task.status != TaskStatus::WaitingApproval {
        return Err(AppError::BadRequest("Task is not waiting for approval".into()));
    }

    if req.approved {
        // Resume the task
        db::update_task_status(&state.pool, id, TaskStatus::Running).await?;
        // Re-send to executor to continue the pipeline
        let _ = state.executor_tx.send(req.run_id).await;
    } else {
        db::update_task_status(&state.pool, id, TaskStatus::Cancelled).await?;
    }

    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Task disappeared".into()))?;
    let runs = db::get_runs_for_task(&state.pool, id).await?;

    Ok(Json(TaskResponse {
        task,
        runs,
        stream_url: None,
    }))
}

/// GET /api/tasks/:id/stream -- SSE for live progress
pub async fn stream_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Sse<ReceiverStream<Result<Event, Infallible>>>, AppError> {
    let task = db::get_task(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    let (progress, sse) = TaskProgressStream::new();

    progress
        .send(ProgressEvent::Starting {
            task_id: task.id.to_string(),
        })
        .await;

    // If task is already completed/failed, send terminal event
    match task.status {
        TaskStatus::Completed => {
            progress
                .send(ProgressEvent::Complete {
                    result_hash: "cached".into(),
                })
                .await;
        }
        TaskStatus::Failed => {
            progress
                .send(ProgressEvent::Failed {
                    message: "Task previously failed".into(),
                })
                .await;
        }
        _ => {
            // Store the progress sender for the executor to use
            let _ = state.progress_tx.send((task.id, progress)).await;
        }
    }

    Ok(sse)
}

/// Parse a cron expression and return the next occurrence.
fn parse_next_cron(cron_expr: &str) -> Option<chrono::DateTime<Utc>> {
    use std::str::FromStr;
    let schedule = cron::Schedule::from_str(cron_expr).ok()?;
    schedule.upcoming(Utc).next()
}

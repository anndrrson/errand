use std::str::FromStr;

use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

use errand_types::TaskKind;

use crate::db;

/// Background scheduler that checks for recurring and monitor tasks due to run.
/// Runs every 30 seconds, creates new TaskRuns, and sends them to the executor.
pub async fn run_scheduler(pool: PgPool, executor_tx: mpsc::Sender<Uuid>) {
    tracing::info!("Scheduler started");

    loop {
        if let Err(e) = tick(&pool, &executor_tx).await {
            tracing::error!("Scheduler tick error: {e}");
        }

        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}

async fn tick(pool: &PgPool, executor_tx: &mpsc::Sender<Uuid>) -> anyhow::Result<()> {
    // 1. Check for due recurring tasks
    let recurring_tasks = db::get_due_recurring_tasks(pool).await?;
    for task in &recurring_tasks {
        tracing::info!("Scheduling recurring task run: {} ({})", task.title, task.id);

        // Create a new run
        let run_id = db::create_task_run(pool, task.id).await?;

        // Send to executor
        let _ = executor_tx.send(run_id).await;

        // Update next_run_at based on cron expression
        if let TaskKind::Recurring { ref cron } = task.kind {
            if let Ok(schedule) = cron::Schedule::from_str(cron) {
                if let Some(next) = schedule.upcoming(Utc).next() {
                    db::update_task_next_run(pool, task.id, next).await?;
                }
            }
        }

        // Deduct credits (2 per recurring run)
        let _ = db::add_credit_transaction(
            pool,
            task.owner_id,
            -2,
            &format!("recurring_run:{}", task.id),
        )
        .await;
    }

    // 2. Check for due monitor tasks
    let monitor_tasks = db::get_due_monitor_tasks(pool).await?;
    for task in &monitor_tasks {
        tracing::info!("Scheduling monitor check: {} ({})", task.title, task.id);

        let run_id = db::create_task_run(pool, task.id).await?;
        let _ = executor_tx.send(run_id).await;

        // Update next_run_at based on check_interval
        if let TaskKind::Monitor {
            check_interval_seconds,
            ..
        } = &task.kind
        {
            let next = Utc::now() + chrono::Duration::seconds(*check_interval_seconds as i64);
            db::update_task_next_run(pool, task.id, next).await?;
        }

        // Deduct credits (1 per monitor check)
        let _ = db::add_credit_transaction(
            pool,
            task.owner_id,
            -1,
            &format!("monitor_check:{}", task.id),
        )
        .await;
    }

    if !recurring_tasks.is_empty() || !monitor_tasks.is_empty() {
        tracing::info!(
            "Scheduler: dispatched {} recurring, {} monitor tasks",
            recurring_tasks.len(),
            monitor_tasks.len()
        );
    }

    Ok(())
}

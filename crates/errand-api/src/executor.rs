use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use errand_types::{RunStatus, TaskKind, TaskStatus};

use crate::agents::AgentOrchestrator;
use crate::config::Config;
use crate::db;
use crate::streaming::{ProgressEvent, TaskProgressStream};

/// Background task executor. Receives task_run IDs and executes them.
pub async fn run_executor(
    pool: PgPool,
    config: Config,
    mut rx: mpsc::Receiver<Uuid>,
    progress_streams: Arc<Mutex<HashMap<Uuid, TaskProgressStream>>>,
    mut progress_rx: mpsc::Receiver<(Uuid, TaskProgressStream)>,
) {
    tracing::info!("Executor started");

    let orchestrator = AgentOrchestrator::new(
        &config.anthropic_api_key,
        &config.tavily_api_key,
        &config.solana_rpc_url,
    );
    let orchestrator = Arc::new(orchestrator);

    loop {
        tokio::select! {
            // Receive new progress stream registrations
            Some((task_id, stream)) = progress_rx.recv() => {
                progress_streams.lock().await.insert(task_id, stream);
            }
            // Receive task run IDs to execute
            Some(run_id) = rx.recv() => {
                let pool = pool.clone();
                let orchestrator = orchestrator.clone();
                let streams = progress_streams.clone();

                tokio::spawn(async move {
                    if let Err(e) = execute_run(&pool, &orchestrator, &streams, run_id).await {
                        tracing::error!("Executor error for run {run_id}: {e}");
                    }
                });
            }
            else => break,
        }
    }
}

async fn execute_run(
    pool: &PgPool,
    orchestrator: &AgentOrchestrator,
    streams: &Arc<Mutex<HashMap<Uuid, TaskProgressStream>>>,
    run_id: Uuid,
) -> anyhow::Result<()> {
    let run = db::get_task_run(pool, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("TaskRun {run_id} not found"))?;

    let task = db::get_task(pool, run.task_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Task {} not found", run.task_id))?;

    // Get progress stream if someone is listening
    let progress = streams.lock().await.remove(&task.id);

    if let Some(ref p) = progress {
        p.send(ProgressEvent::Starting {
            task_id: task.id.to_string(),
        })
        .await;
    }

    match &task.kind {
        TaskKind::OneShot | TaskKind::Recurring { .. } => {
            execute_oneshot(pool, orchestrator, &progress, &task, run_id).await?;
        }
        TaskKind::Monitor { .. } => {
            execute_monitor(pool, orchestrator, &progress, &task, run_id).await?;
        }
        TaskKind::Pipeline { steps } => {
            execute_pipeline(pool, orchestrator, &progress, &task, run_id, steps).await?;
        }
    }

    // Send webhook notification if configured
    if let Some(ref webhook_url) = task.webhook_url {
        if let Err(e) = send_webhook(webhook_url, &task.id, &run_id).await {
            tracing::warn!("Webhook failed for task {} (attempt 1): {e}", task.id);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            if let Err(e2) = send_webhook(webhook_url, &task.id, &run_id).await {
                tracing::warn!("Webhook failed for task {} (attempt 2): {e2}", task.id);
            }
        }
    }

    Ok(())
}

async fn execute_oneshot(
    pool: &PgPool,
    orchestrator: &AgentOrchestrator,
    progress: &Option<TaskProgressStream>,
    task: &errand_types::Task,
    run_id: Uuid,
) -> anyhow::Result<()> {
    if let Some(ref p) = progress {
        p.send(ProgressEvent::Searching {
            query: task.title.clone(),
        })
        .await;
    }

    match orchestrator.execute_task(task).await {
        Ok(result) => {
            if let Some(ref p) = progress {
                p.send(ProgressEvent::Writing {
                    section: "Finalizing result...".into(),
                })
                .await;
            }

            db::update_task_run_completed(
                pool,
                run_id,
                RunStatus::Completed,
                Some(&result.result_text),
                Some(&result.result_hash),
                1,
                1,
            )
            .await?;

            // For one-shot tasks, mark the task as completed
            if matches!(task.kind, TaskKind::OneShot) {
                db::update_task_status(pool, task.id, TaskStatus::Completed).await?;
            }

            if let Some(ref p) = progress {
                p.send(ProgressEvent::Complete {
                    result_hash: result.result_hash,
                })
                .await;
            }
        }
        Err(e) => {
            tracing::error!("Agent execution failed for task {}: {e}", task.id);

            db::update_task_run_completed(
                pool,
                run_id,
                RunStatus::Failed,
                Some(&format!("Error: {e}")),
                None,
                0,
                0,
            )
            .await?;

            if matches!(task.kind, TaskKind::OneShot) {
                db::update_task_status(pool, task.id, TaskStatus::Failed).await?;
            }

            if let Some(ref p) = progress {
                p.send(ProgressEvent::Failed {
                    message: format!("Agent execution failed: {e}"),
                })
                .await;
            }
        }
    }

    Ok(())
}

async fn execute_monitor(
    pool: &PgPool,
    orchestrator: &AgentOrchestrator,
    progress: &Option<TaskProgressStream>,
    task: &errand_types::Task,
    run_id: Uuid,
) -> anyhow::Result<()> {
    if let Some(ref p) = progress {
        p.send(ProgressEvent::Analyzing {
            step: "Checking condition...".into(),
        })
        .await;
    }

    match orchestrator.check_condition(task).await {
        Ok((met, evidence)) => {
            let result_text = if met {
                format!("CONDITION MET: {evidence}")
            } else {
                format!("CONDITION NOT MET: {evidence}")
            };

            db::update_task_run_completed(
                pool,
                run_id,
                RunStatus::Completed,
                Some(&result_text),
                None,
                1,
                1,
            )
            .await?;

            if let Some(ref p) = progress {
                if met {
                    p.send(ProgressEvent::ConditionMet { evidence }).await;
                } else {
                    p.send(ProgressEvent::ConditionNotMet {
                        summary: evidence,
                    })
                    .await;
                }
            }

            // If condition is met, complete the monitor task
            if met {
                db::update_task_status(pool, task.id, TaskStatus::Completed).await?;
            }
        }
        Err(e) => {
            db::update_task_run_completed(
                pool,
                run_id,
                RunStatus::Failed,
                Some(&format!("Monitor check error: {e}")),
                None,
                0,
                0,
            )
            .await?;

            if let Some(ref p) = progress {
                p.send(ProgressEvent::Failed {
                    message: format!("Monitor check failed: {e}"),
                })
                .await;
            }
        }
    }

    Ok(())
}

async fn execute_pipeline(
    pool: &PgPool,
    orchestrator: &AgentOrchestrator,
    progress: &Option<TaskProgressStream>,
    task: &errand_types::Task,
    run_id: Uuid,
    steps: &[errand_types::PipelineStep],
) -> anyhow::Result<()> {
    // Get current progress (in case we're resuming after approval)
    let current_run = db::get_task_run(pool, run_id).await?;
    let start_step = current_run.map(|r| r.steps_completed as usize).unwrap_or(0);

    let mut accumulated_results = String::new();

    for (i, step) in steps.iter().enumerate().skip(start_step) {
        if let Some(ref p) = progress {
            p.send(ProgressEvent::Analyzing {
                step: format!("Step {}: {}", i + 1, step.description),
            })
            .await;
        }

        // Check if this step requires approval
        if step.requires_approval && i == start_step && start_step > 0 {
            // We're resuming after approval, skip the approval check
        } else if step.requires_approval && i > 0 {
            // Pause for human approval
            db::update_task_run_steps(pool, run_id, i as u32).await?;
            db::update_task_status(pool, task.id, TaskStatus::WaitingApproval).await?;

            if let Some(ref p) = progress {
                p.send(ProgressEvent::WaitingApproval {
                    step_index: i as u32,
                    description: step.description.clone(),
                })
                .await;
            }
            return Ok(());
        }

        // Create a sub-task description that includes context from previous steps
        let step_description = if accumulated_results.is_empty() {
            format!(
                "{}\n\nCurrent step: {}",
                task.description, step.description
            )
        } else {
            format!(
                "{}\n\nPrevious results:\n{}\n\nCurrent step: {}",
                task.description, accumulated_results, step.description
            )
        };

        // Create a temporary task for this step
        let step_task = errand_types::Task {
            id: task.id,
            owner_id: task.owner_id,
            title: format!("{} - Step {}", task.title, i + 1),
            description: step_description,
            kind: TaskKind::OneShot,
            category: task.category,
            status: task.status,
            webhook_url: None,
            email_notify: None,
            created_at: task.created_at,
            updated_at: task.updated_at,
            next_run_at: None,
        };

        match orchestrator.execute_task(&step_task).await {
            Ok(result) => {
                accumulated_results
                    .push_str(&format!("\n--- Step {} ---\n{}\n", i + 1, result.result_text));

                db::update_task_run_steps(pool, run_id, (i + 1) as u32).await?;

                if let Some(ref p) = progress {
                    p.send(ProgressEvent::StepComplete {
                        step_index: i as u32,
                        summary: format!(
                            "Completed: {}",
                            &result.result_text[..result.result_text.len().min(100)]
                        ),
                    })
                    .await;
                }
            }
            Err(e) => {
                db::update_task_run_completed(
                    pool,
                    run_id,
                    RunStatus::Failed,
                    Some(&format!("Pipeline failed at step {}: {e}", i + 1)),
                    None,
                    steps.len() as u32,
                    i as u32,
                )
                .await?;

                db::update_task_status(pool, task.id, TaskStatus::Failed).await?;

                if let Some(ref p) = progress {
                    p.send(ProgressEvent::Failed {
                        message: format!("Pipeline failed at step {}: {e}", i + 1),
                    })
                    .await;
                }
                return Ok(());
            }
        }
    }

    // All steps completed
    let hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(accumulated_results.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    db::update_task_run_completed(
        pool,
        run_id,
        RunStatus::Completed,
        Some(&accumulated_results),
        Some(&hash),
        steps.len() as u32,
        steps.len() as u32,
    )
    .await?;

    db::update_task_status(pool, task.id, TaskStatus::Completed).await?;

    if let Some(ref p) = progress {
        p.send(ProgressEvent::Complete {
            result_hash: hash,
        })
        .await;
    }

    Ok(())
}

async fn send_webhook(url: &str, task_id: &Uuid, run_id: &Uuid) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "event": "task_run_complete",
        "task_id": task_id,
        "run_id": run_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    client
        .post(url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    Ok(())
}

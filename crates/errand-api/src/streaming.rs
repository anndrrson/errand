use axum::response::sse::{Event, Sse};
use serde::Serialize;
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;

/// Progress event types sent during task execution.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ProgressEvent {
    Starting { task_id: String },
    Searching { query: String },
    Analyzing { step: String },
    Writing { section: String },
    WaitingApproval { step_index: u32, description: String },
    StepComplete { step_index: u32, summary: String },
    Complete { result_hash: String },
    Failed { message: String },
    ConditionMet { evidence: String },
    ConditionNotMet { summary: String },
}

/// A channel-based SSE stream for task progress updates.
pub struct TaskProgressStream {
    tx: tokio::sync::mpsc::Sender<ProgressEvent>,
}

impl TaskProgressStream {
    /// Create a new progress stream, returning the stream handle and an SSE response.
    pub fn new() -> (Self, Sse<ReceiverStream<Result<Event, Infallible>>>) {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel::<ProgressEvent>(32);
        let (sse_tx, sse_rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(32);

        // Bridge progress events to SSE events
        tokio::spawn(async move {
            let mut rx = event_rx;
            while let Some(event) = rx.recv().await {
                let event_name = match &event {
                    ProgressEvent::Starting { .. } => "starting",
                    ProgressEvent::Searching { .. } => "searching",
                    ProgressEvent::Analyzing { .. } => "analyzing",
                    ProgressEvent::Writing { .. } => "writing",
                    ProgressEvent::WaitingApproval { .. } => "waiting_approval",
                    ProgressEvent::StepComplete { .. } => "step_complete",
                    ProgressEvent::Complete { .. } => "complete",
                    ProgressEvent::Failed { .. } => "failed",
                    ProgressEvent::ConditionMet { .. } => "condition_met",
                    ProgressEvent::ConditionNotMet { .. } => "condition_not_met",
                };

                let data = serde_json::to_string(&event).unwrap_or_default();
                let sse_event = Event::default().event(event_name).data(data);

                if sse_tx.send(Ok(sse_event)).await.is_err() {
                    break;
                }

                // Terminal events end the stream
                if matches!(
                    event,
                    ProgressEvent::Complete { .. }
                        | ProgressEvent::Failed { .. }
                        | ProgressEvent::WaitingApproval { .. }
                        | ProgressEvent::ConditionMet { .. }
                ) {
                    break;
                }
            }
        });

        let stream = ReceiverStream::new(sse_rx);
        let sse = Sse::new(stream);

        (Self { tx: event_tx }, sse)
    }

    /// Send a progress event.
    pub async fn send(&self, event: ProgressEvent) {
        let _ = self.tx.send(event).await;
    }
}

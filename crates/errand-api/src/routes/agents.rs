use axum::{extract::State, Json};

use crate::{db, error::AppError, AppState};
use errand_types::AgentListResponse;

/// GET /api/agents -- list available agents
pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<AgentListResponse>, AppError> {
    let agents = db::list_agents(&state.pool).await?;
    Ok(Json(AgentListResponse { agents }))
}

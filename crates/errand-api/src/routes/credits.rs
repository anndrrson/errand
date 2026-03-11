use axum::{extract::State, Json};

use crate::{auth::AuthUser, db, error::AppError, AppState};
use errand_types::{CreditBalanceResponse, CreditHistoryResponse, RedeemCodeRequest};

/// GET /api/credits/balance -- get credit balance
pub async fn get_balance(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<CreditBalanceResponse>, AppError> {
    let balance = db::get_credit_balance(&state.pool, auth.user_id).await?;
    Ok(Json(CreditBalanceResponse { balance }))
}

/// GET /api/credits/history -- credit transaction history
pub async fn get_history(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<CreditHistoryResponse>, AppError> {
    let transactions = db::get_credit_history(&state.pool, auth.user_id).await?;
    Ok(Json(CreditHistoryResponse { transactions }))
}

/// POST /api/credits/redeem -- redeem a beta invite code
pub async fn redeem_code(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<RedeemCodeRequest>,
) -> Result<Json<CreditBalanceResponse>, AppError> {
    // Simple beta codes: ERRAND-BETA-{anything} = 20 credits
    if !req.code.starts_with("ERRAND-BETA-") {
        return Err(AppError::BadRequest("Invalid redemption code".into()));
    }

    db::add_credit_transaction(&state.pool, auth.user_id, 20, &format!("redeem:{}", req.code))
        .await?;

    let balance = db::get_credit_balance(&state.pool, auth.user_id).await?;
    Ok(Json(CreditBalanceResponse { balance }))
}

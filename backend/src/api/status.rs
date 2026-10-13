use axum::Json;
use axum::extract::State;

use super::AppState;
use super::dto::{StatusResponse, StatusSummaryResponse};
use super::error::ApiError;

/// `GET /api/status` : évacués et manquants, pour l'écran de l'agent.
pub async fn handle_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, ApiError> {
    let report = state.evacuation.report().await?;

    Ok(Json(StatusResponse::from(report)))
}

/// `GET /api/status/summary` : les mêmes informations en compteurs.
pub async fn handle_summary(
    State(state): State<AppState>,
) -> Result<Json<StatusSummaryResponse>, ApiError> {
    let summary = state.evacuation.summary().await?;

    Ok(Json(StatusSummaryResponse::from(summary)))
}

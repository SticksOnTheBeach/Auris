use axum::Json;
use axum::extract::State;

use super::AppState;
use super::dto::{ScanRequest, ScanResponse};
use super::error::ApiError;

/// `POST /api/scan` : pointage d'un badge par une borne.
///
/// Le handler ne décide rien. Il traduit la requête HTTP en appel métier,
/// puis le résultat métier en réponse HTTP.
pub async fn handle_scan(
    State(state): State<AppState>,
    Json(request): Json<ScanRequest>,
) -> Result<Json<ScanResponse>, ApiError> {
    println!(
        "scan : borne={} badge={}",
        request.borne_id, request.badge_hash
    );

    let status = state
        .evacuation
        .register_scan(&request.borne_id, &request.badge_hash)
        .await?;

    Ok(Json(ScanResponse::from(status)))
}

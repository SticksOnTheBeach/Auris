use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::service::ServiceError;

/// Traduction d'une erreur métier en réponse HTTP.
///
/// Les codes de statut vivent ici et nulle part ailleurs : le service ignore
/// qu'il est exposé en HTTP.
pub struct ApiError(ServiceError);

#[derive(Serialize)]
struct ApiErrorBody {
    error: &'static str,
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let Self(error) = self;

        // le détail reste dans les logs du serveur, la borne ne reçoit qu'un code
        eprintln!("Error : {}", error);

        let (code, body) = match error {
            ServiceError::Storage(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiErrorBody {
                    error: "storage_unavailable",
                },
            ),
        };

        (code, Json(body)).into_response()
    }
}

//! La couche HTTP : routage, formats d'échange et traduction des erreurs.
//! C'est la seule couche qui connaît axum. Elle ne contient aucune règle
//! métier : chaque handler délègue au service.

pub mod dto;
pub mod error;
mod scan;
mod status;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::service::EvacuationService;

/// État partagé par toutes les requêtes.
///
/// Il n'expose que le service métier : un handler ne peut pas toucher au
/// stockage directement, même par erreur.
#[derive(Clone)]
pub struct AppState {
    pub evacuation: Arc<EvacuationService>,
}

impl AppState {
    pub fn new(evacuation: Arc<EvacuationService>) -> Self {
        Self { evacuation }
    }
}

/// Table de routage d'Auris. Ajouter une route (le futur WebSocket `/ws`,
/// le pointage manuel) ne demande pas de modifier les handlers existants.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/scan", post(scan::handle_scan))
        .route("/api/status", get(status::handle_status))
        .route("/api/status/summary", get(status::handle_summary))
        .with_state(state)
}

//! Le coeur métier : les notions d'Auris, sans aucune dépendance à HTTP,
//! à la base de données ou à axum. Cette couche ne dépend de rien.

pub mod badge;
pub mod evacuation;
pub mod scan;

pub use badge::{BadgeHash, BorneId};
pub use evacuation::{EvacuationReport, EvacuationSummary};
pub use scan::ScanStatus;

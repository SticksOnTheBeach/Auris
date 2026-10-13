//! Formats d'échange avec la borne et le frontend.
//!
//! Ils sont volontairement distincts des types du domaine : renommer un champ
//! JSON du contrat d'API ne touche que ce fichier, et un remaniement interne
//! du domaine ne casse pas le contrat.

use serde::{Deserialize, Serialize};

use crate::domain::{BadgeHash, BorneId, EvacuationReport, EvacuationSummary, ScanStatus};

/// Corps envoyé par la borne sur `POST /api/scan`.
#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub borne_id: BorneId,
    pub badge_hash: BadgeHash,
}

/// Accusé de réception renvoyé à la borne.
#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub status: ScanStatus,
}

impl From<ScanStatus> for ScanResponse {
    fn from(status: ScanStatus) -> Self {
        Self { status }
    }
}

/// Réponse de `GET /api/status` : les deux listes et l'effectif.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub total: usize,
    pub evacuate_person: Vec<BadgeHash>,
    pub missing_person: Vec<BadgeHash>,
}

impl From<EvacuationReport> for StatusResponse {
    fn from(report: EvacuationReport) -> Self {
        Self {
            total: report.total,
            evacuate_person: report.evacuated,
            missing_person: report.missing,
        }
    }
}

/// Réponse de `GET /api/status/summary` : les compteurs seuls.
#[derive(Debug, Serialize)]
pub struct StatusSummaryResponse {
    pub total: usize,
    pub evacuate_person: usize,
    pub missing_person: usize,
}

impl From<EvacuationSummary> for StatusSummaryResponse {
    fn from(summary: EvacuationSummary) -> Self {
        Self {
            total: summary.total,
            evacuate_person: summary.evacuated,
            missing_person: summary.missing,
        }
    }
}

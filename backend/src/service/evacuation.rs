use std::collections::HashSet;
use std::sync::Arc;

use thiserror::Error;

use crate::domain::{BadgeHash, BorneId, EvacuationReport, EvacuationSummary, ScanStatus};
use crate::repository::{EmployeeRepository, RepositoryError, ScanRepository};

/// Échec d'une opération métier.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("accès aux données impossible : {0}")]
    Storage(#[from] RepositoryError),
}

/// Toute l'intelligence du serveur.
///
/// Il ne connaît ni axum, ni MySQL : il ne parle qu'aux deux interfaces de
/// dépôt, qui lui sont injectées à la construction. C'est ce qui permet de le
/// tester avec de faux dépôts, sans serveur HTTP ni base de données.
pub struct EvacuationService {
    employees: Arc<dyn EmployeeRepository>,
    scans: Arc<dyn ScanRepository>,
}

impl EvacuationService {
    pub fn new(employees: Arc<dyn EmployeeRepository>, scans: Arc<dyn ScanRepository>) -> Self {
        Self { employees, scans }
    }

    /// Règle du pointage : un badge inconnu est rejeté, un badge connu est
    /// enregistré, un badge déjà vu est signalé comme doublon.
    pub async fn register_scan(
        &self,
        borne: &BorneId,
        badge: &BadgeHash,
    ) -> Result<ScanStatus, ServiceError> {
        if !self.employees.contains(badge).await? {
            return Ok(ScanStatus::Unknown);
        }

        if self.scans.record(borne, badge).await? {
            Ok(ScanStatus::Ok)
        } else {
            Ok(ScanStatus::AlreadyScanned)
        }
    }

    /// Croise les employés attendus et les badges pointés pour obtenir les
    /// deux listes affichées à l'agent.
    pub async fn report(&self) -> Result<EvacuationReport, ServiceError> {
        let known: HashSet<BadgeHash> = self.employees.badges().await?.into_iter().collect();
        let scanned: HashSet<BadgeHash> = self.scans.scanned().await?.into_iter().collect();

        let mut evacuated: Vec<BadgeHash> = scanned.iter().cloned().collect();
        let mut missing: Vec<BadgeHash> = known.difference(&scanned).cloned().collect();

        // un ensemble n'a pas d'ordre : on trie pour des réponses stables
        evacuated.sort();
        missing.sort();

        Ok(EvacuationReport {
            total: known.len(),
            evacuated,
            missing,
        })
    }

    /// Mêmes informations que `report`, en compteurs seulement : deux
    /// `count()` suffisent, sans charger les listes.
    pub async fn summary(&self) -> Result<EvacuationSummary, ServiceError> {
        let total = self.employees.count().await?;
        let evacuated = self.scans.count().await?;

        Ok(EvacuationSummary::new(total, evacuated))
    }
}

//! Accès aux données, défini par des interfaces (traits) et non par une
//! implémentation. C'est le pivot de l'inversion de dépendance : le métier
//! dépend des traits ci-dessous, pas du `HashSet` ni, demain, de sqlx.

pub mod memory;

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::{BadgeHash, BorneId};

/// Panne de stockage. Le métier n'a pas à savoir s'il s'agit d'un `Mutex`
/// empoisonné ou d'une connexion MySQL tombée.
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("stockage indisponible : {0}")]
    Unavailable(String),
}

/// Lecture de la base des employés.
///
/// Deux interfaces séparées plutôt qu'une seule (ségrégation des interfaces) :
/// la consultation du statut n'a pas besoin du droit d'écrire un pointage.
#[async_trait]
pub trait EmployeeRepository: Send + Sync {
    /// Le badge appartient-il à un employé enregistré ?
    async fn contains(&self, badge: &BadgeHash) -> Result<bool, RepositoryError>;

    /// Tous les badges attendus à l'appel.
    async fn badges(&self) -> Result<Vec<BadgeHash>, RepositoryError>;

    /// Effectif total, sans charger la liste complète.
    async fn count(&self) -> Result<usize, RepositoryError>;
}

/// Écriture et lecture des pointages de l'exercice en cours.
#[async_trait]
pub trait ScanRepository: Send + Sync {
    /// Enregistre le passage d'un badge à une borne et renvoie `true` si
    /// c'était le premier pointage de ce badge pour cet exercice.
    async fn record(&self, borne: &BorneId, badge: &BadgeHash) -> Result<bool, RepositoryError>;

    /// Les badges déjà pointés.
    async fn scanned(&self) -> Result<Vec<BadgeHash>, RepositoryError>;

    /// Nombre de badges pointés.
    async fn count(&self) -> Result<usize, RepositoryError>;
}

//! Implémentation en mémoire des deux dépôts, le temps que MySQL arrive
//! (étape 1 de la feuille de route). Remplacer ce fichier par un
//! `repository/mysql.rs` ne demandera aucune modification du métier ni des
//! handlers : seul le câblage de `main.rs` changera.

use std::collections::HashSet;
use std::sync::Mutex;

use async_trait::async_trait;

use super::{EmployeeRepository, RepositoryError, ScanRepository};
use crate::domain::{BadgeHash, BorneId};

/// Base des employés en dur.
pub struct InMemoryEmployees {
    badges: HashSet<BadgeHash>,
}

impl InMemoryEmployees {
    pub fn new(badges: impl IntoIterator<Item = BadgeHash>) -> Self {
        Self {
            badges: badges.into_iter().collect(),
        }
    }

    /// Jeu de test : les trois badges fictifs utilisés depuis le début.
    /// Aucun vrai UID d'employé ne doit atterrir ici.
    pub fn with_fake_employees() -> Self {
        Self::new(
            ["hash_alice", "hash_bob", "hash_charlie"]
                .into_iter()
                .map(BadgeHash::new),
        )
    }
}

#[async_trait]
impl EmployeeRepository for InMemoryEmployees {
    async fn contains(&self, badge: &BadgeHash) -> Result<bool, RepositoryError> {
        Ok(self.badges.contains(badge))
    }

    async fn badges(&self) -> Result<Vec<BadgeHash>, RepositoryError> {
        Ok(self.badges.iter().cloned().collect())
    }

    async fn count(&self) -> Result<usize, RepositoryError> {
        Ok(self.badges.len())
    }
}

/// Pointages de l'exercice en cours.
pub struct InMemoryScans {
    scanned: Mutex<HashSet<BadgeHash>>,
}

impl InMemoryScans {
    pub fn new() -> Self {
        Self {
            scanned: Mutex::new(HashSet::new()),
        }
    }

    /// Un `Mutex` empoisonné par un panic antérieur ne doit pas rendre le
    /// serveur muet en pleine évacuation : on récupère la donnée protégée.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashSet<BadgeHash>> {
        self.scanned
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for InMemoryScans {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScanRepository for InMemoryScans {
    async fn record(&self, _borne: &BorneId, badge: &BadgeHash) -> Result<bool, RepositoryError> {
        // La borne d'origine n'est pas encore stockée : il faudra une table
        // `scans` pour distinguer entrée du bâtiment et point de rassemblement.
        Ok(self.lock().insert(badge.clone()))
    }

    async fn scanned(&self) -> Result<Vec<BadgeHash>, RepositoryError> {
        Ok(self.lock().iter().cloned().collect())
    }

    async fn count(&self) -> Result<usize, RepositoryError> {
        Ok(self.lock().len())
    }
}

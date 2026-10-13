use std::fmt;

use serde::{Deserialize, Serialize};

/// Empreinte SHA-256 d'un badge.
///
/// Le serveur ne manipule jamais l'UID brut : la borne le hashe avant de
/// l'envoyer. Le type enveloppe une `String` pour qu'un hash de badge ne
/// puisse pas être confondu avec un identifiant de borne.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BadgeHash(String);

impl BadgeHash {
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BadgeHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifiant d'une borne (`ext_nord`, `entree_hall`...).
///
/// Il sert à distinguer une borne d'entrée dans le bâtiment d'une borne de
/// point de rassemblement : c'est ce qui permettra de croiser les deux.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BorneId(String);

impl BorneId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BorneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

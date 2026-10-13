use super::badge::BadgeHash;

/// Photographie de l'évacuation : qui est sorti, qui manque.
///
/// Les listes sont triées pour que deux appels successifs renvoient le même
/// ordre (un ensemble de hashs n'a pas d'ordre naturel).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvacuationReport {
    pub total: usize,
    pub evacuated: Vec<BadgeHash>,
    pub missing: Vec<BadgeHash>,
}

/// Les mêmes données réduites à des compteurs, pour l'affichage temps réel
/// sur la tablette de l'agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvacuationSummary {
    pub total: usize,
    pub evacuated: usize,
    pub missing: usize,
}

impl EvacuationSummary {
    /// `saturating_sub` et non `-` : si le nombre de pointages dépassait
    /// l'effectif connu (liste d'employés modifiée en pleine évacuation),
    /// une soustraction classique paniquerait en debug.
    pub fn new(total: usize, evacuated: usize) -> Self {
        Self {
            total,
            evacuated,
            missing: total.saturating_sub(evacuated),
        }
    }
}

impl From<&EvacuationReport> for EvacuationSummary {
    fn from(report: &EvacuationReport) -> Self {
        Self::new(report.total, report.evacuated.len())
    }
}

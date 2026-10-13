use serde::Serialize;

/// Résultat métier d'un pointage.
///
/// Les trois seuls cas que la borne peut recevoir, tels que figés dans
/// `docs/api-contrat.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    /// Badge connu, évacuation enregistrée.
    Ok,
    /// Badge absent de la base des employés.
    Unknown,
    /// Badge déjà pointé pendant cet exercice.
    AlreadyScanned,
}

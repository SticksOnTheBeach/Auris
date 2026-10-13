//! Les règles métier. Cette couche décide ; les handlers HTTP ne font que
//! l'appeler et les dépôts ne font que stocker.

pub mod evacuation;

pub use evacuation::{EvacuationService, ServiceError};

//! Point d'entrée d'Auris : le serveur de pointage d'évacuation.
//!
//! Ce fichier ne contient ni règle métier, ni requête de stockage, ni routage :
//! il assemble les couches de la bibliothèque et démarre le serveur. C'est le
//! seul endroit du programme qui connaît les implémentations concrètes des
//! dépôts.

use std::process::ExitCode;
use std::sync::Arc;

use auris::api::{self, AppState};
use auris::config::ServerConfig;
use auris::repository::memory::{InMemoryEmployees, InMemoryScans};
use auris::service::EvacuationService;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match ServerConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Error : configuration invalide ({})", error);
            return ExitCode::FAILURE;
        }
    };

    // Câblage des dépendances : brancher MySQL plus tard ne changera que ces
    // deux lignes, le service et les handlers resteront intacts.
    let employees = Arc::new(InMemoryEmployees::with_fake_employees());
    let scans = Arc::new(InMemoryScans::new());

    let evacuation = Arc::new(EvacuationService::new(employees, scans));
    let state = AppState::new(evacuation);

    let listener = match tokio::net::TcpListener::bind(config.address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Error : failed to bind {} ({})", config.address, error);
            return ExitCode::FAILURE;
        }
    };

    println!("Server listening at : http://{}/api/scan", config.address);

    if let Err(error) = axum::serve(listener, api::router(state)).await {
        eprintln!("Error : server STOPPED ! : {}", error);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

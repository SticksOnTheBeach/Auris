use axum::{extract::State, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};


// On définit la structure des données que le C++ vas envoyer à /Api/scan
// L'attribut `#[derive(Deserialize)]` permet à Axum de transformer le JSON reçu en cette structure.
#[derive(Deserialize, Debug)]
struct scanInfos {
    borne_id: String,
    badge_hash: String,
}

/*
* établissement d'une énum avec les différents status qui peuvent être renvoyés.
* Ce que le serveur répond
*/
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum ScanStatus {
    Ok,
    Unknown,
    AlreadyScanned,
}


#[derive(Serialize)]
struct ScanResponse {
    status: ScanStatus,
}

#[derive(Serialize)]
struct StatusResponse {
    total: usize,
    evacuate_person: Vec<String>,
    missing_person: Vec<String>,
}

#[derive(Serialize)]
struct StatusNumberResponse {
    total: usize,
    evacuate_person : Vec<usize>,
    missing_person : Vec<usize>,
}

// shared states between all the requests
#[derive(Clone)]
struct AppState {
    known: Arc<HashSet<String>>, // knew badges ( replace the DB )
    scanned: Arc<Mutex<HashSet<String>>>, // badges already scanned
}


#[tokio::main]
async fn main() {
    // instanciation de badge connu : permet de remplacer la bdd
    // instanciation of badges to replace the db system
    let known: HashSet<String> = ["hash_alice", "hash_bob", "hash_charlie"].into_iter().map(String::from).collect();

    let state = AppState {
        known: Arc::new(known),
        scanned: Arc::new(Mutex::new(HashSet::new())),
    };

    // 2. On configure la route /api/scan pour accepter uniquement les requêtes POST
    let api = Router::new().route("/api/scan", post(handle_scan)).route("/api/status", get(handle_status)).with_state(state);
    // Écoute sur toutes les interfaces réseau (0.0.0.0) sur le port 8080
    let firmware_serveur_endpoint = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server listening at : http://localhost:8080/api/scan");


    let listener =  match tokio::net::TcpListener::bind(firmware_serveur_endpoint).await {
        Ok(listener) => {
            println!("Connected to the server at {}", firmware_serveur_endpoint);
            listener
        },
        Err(error) => {
            eprintln!("Error : failed to bind {} ({})", firmware_serveur_endpoint, error);
            std::process::exit(1);
        }
    };

    if let Err(error) = axum::serve(listener, api).await {
        eprintln!("Error : server STOPPED ! : {}", error);
        std::process::exit(1);
    }

}

// 3. Le "handler" qui reçoit les données
// Axum extrait automatiquement le corps de la requête grâce à `Json(payload)`
async fn handle_scan(State(state): State<AppState>, Json(server_infos): Json<scanInfos>) -> Json<ScanResponse> {
    println!("scan : {:?}", server_infos);

    let status = if !state.known.contains(&server_infos.badge_hash) {
        ScanStatus::Unknown
    } else {
        let mut scanned = state.scanned.lock().unwrap_or_else(|e| e.into_inner());
        // insert() renvoie true si le badge n'y était pas encore
        if scanned.insert(server_infos.badge_hash) {
            ScanStatus::Ok
        } else {
            ScanStatus::AlreadyScanned
        }
    };

    Json(ScanResponse { status })
}

async fn handle_status(State(state): State<AppState>) -> Json<StatusResponse>  {
    let scanned = state.scanned.lock().unwrap_or_else(|e| e.into_inner());

    let mut evacuate_person: Vec<String> = scanned.iter().cloned().collect();
    let mut missing_person : Vec<String> = state.known.difference(&scanned).cloned().collect();
    

    // un HashSet n'a pas d'ordre : on trie pour avoir des réponses stables
    evacuate_person.sort();
    missing_person.sort();

    Json(StatusResponse {
        total: state.known.len(),
        evacuate_person,
        missing_person,
    })
}
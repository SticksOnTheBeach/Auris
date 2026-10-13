use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

// On définit la structure des données que le C++ vas envoyer à /Api/scan
// L'attribut `#[derive(Deserialize)]` permet à Axum de transformer le JSON reçu en cette structure.
#[derive(Deserialize, Debug)]
struct ServerInformations {
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
    let mut badges = Arc::new(String::from);
    let known: HashSet<String> = ["hash_alice", "hash_bob", "hash_charlie"].into_iter().map(*badges).collect();

    let state = AppState {
        known: Arc::new(known),
        scanned: Arc::new(Mutex::new(HashSet::new())),
    };

    // 2. On configure la route /api/post pour accepter uniquement les requêtes POST
    let api = Router::new().route("/api/post", post(handle_scan));

    // Écoute sur toutes les interfaces réseau (0.0.0.0) sur le port 3000
    let firmware_serveur_endpoint = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening at : http://localhost:3000/api/post");


    let listener =  match tokio::net::TcpListener::bind(firmware_serveur_endpoint).await {
        Ok(listener) => {
            println!("Connected to the server at {}", firmware_serveur_endpoint);
            listener
        },
        Err(error) => {
            eprintln!("Error : failed to connect at {} ({})", firmware_serveur_endpoint, error);
            std::process::exit(1);
        }
    };

    axum::serve(listener, api).await.unwrap();
}

// 3. Le "handler" qui reçoit les données
// Axum extrait automatiquement le corps de la requête grâce à `Json(payload)`
async fn handle_scan(Json(server_infos): Json<ServerInformations>) -> Json<ScanResponse> {
    println!("scan : {:?}", server_infos);

    Json(ScanResponse { status: ScanStatus::Ok })
}

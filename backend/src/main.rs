/*
*
*
* So, i just made this branch to get a better view of what am i coding.
* In the others branch, i was lost about what am i really doing, and i was thinking that using Claude
* all the time i encountered a issues was not a good idea. By using Claude, i couldn't learnt anything
* about how to fix the issues or stuff like that.
*
* Ver. 0.0.4
* Release : 10/15/26 - Thursday, 14.38h
* Loc. Unknow.
*/
use std::collections::HashSet;
use axum::{
    Router,
    extract::Json,
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};
use axum::extract::State;
use axum::routing::get;

// struct that just represented the data send at the /api/scan route
#[derive(Deserialize, Debug)]
struct ScanPayload {
    borne_id: String,
    badge_hash: String,
}

// What the server answers to the scanner
#[derive(Serialize,Debug)]
#[serde(rename_all = "snake_case")] // rename all the string in snake case
enum ScanStatus {
    Ok,
    Unknown,
    AlreadyScanned,
}
// send to the server status that has scanStatus type
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
struct StatusResponseSummary {
    total: usize,
    evacuate_person_count: usize,
    missing_person_count: usize,
}

/* So, if you're new :
*
*
*   explanationOfTheProject.txt
*
 */
#[tokio::main]
async fn main() {
    // initialize tracing -> returning log informations
    tracing_subscriber::fmt::init();

    // load the env file
    dotenvy::dotenv().ok();

    let listening_address =
        env::var("APP_LISTENING_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    // instanciation of a counter to know if a badge is already scanned or not
    let scanned : Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    // build our application with a route
    let app = Router::new()
        .route("/api/scan", post(handle_scan))
        .route("/api/status", get(handle_status))
        .route("/api/status/summary", get(handle_status_summary)
        ).with_state(scanned);

    // run our app with hyper, listening globally on port 3000
    //let listener = tokio::net::TcpListener::bind(&listening_address).await.unwrap();

    let listener = match tokio::net::TcpListener::bind(&listening_address).await {
        Ok(listener) => {
            listener
        }
        Err(error) => {
            eprintln!("Error: failed to bind {} ({})", listening_address, error);
            std::process::exit(1);
        }
    };
    //println!("Serveur listening to... http://{}", &listener);
    tracing::info!("Server listening at http://{}/api/scan", listening_address);

    if let Err(error) = axum::serve(listener, app).await {
        eprintln!("Error: server stopped ({})", error);
        std::process::exit(1);
    }

    // axum::serve(listener, app).await;
}


const KNOWN_BADGES: [&str; 3] = [
    "84610f4c410a5cbfbb8b9ceb949a42a5b9f7aa7c258e6c68d479f19660a2b014", // Alice
    "de8f7894420153e4c90d5d1d7e3f81c6fd0f3341506dfaafaaba8e31baa51a77", // Bob
    "807eb7f161032fdf89ddc11f4d9c75ca082558b965e3cfc1e40b2bea20a8e420", // Chloé
];

// handler that just retrieves and processes the data
async fn handle_scan(State(scanned) : State<Arc<Mutex<HashSet<String>>>>, Json(payload): Json<ScanPayload>) -> Json<ScanResponse> {
    let status = if KNOWN_BADGES.contains(&payload.badge_hash.as_str()) {
        let first_time = scanned.lock().unwrap().insert(payload.badge_hash.clone());
        if first_time {
            ScanStatus::Ok
        } else {
            ScanStatus::AlreadyScanned
        }
    } else {
        ScanStatus::Unknown
    };

    tracing::info!(
        borne_id = %payload.borne_id,
        badge = %payload.badge_hash.chars().take(8).collect::<String>(),
        status = ?status,
        "Scan processed"
    );

    Json(ScanResponse { status })
}

async fn handle_status(State(scanned): State<Arc<Mutex<HashSet<String>>>>)  -> Json<StatusResponse> {
    let scanned = scanned.lock().unwrap();

    let mut evacuate_person: Vec<String> = scanned.iter().cloned().collect();

    /*there's two ways to have all of the missing person, or in a common way the non-scanned people */
    /*let mut missing_person: Vec<String> = Vec::new();

    for badge in KNOWN_BADGES.iter() {
        if !scanned.contains(*badge) {
            missing_person.push(badge.to_string());
        }
    }*/

    let mut missing_person: Vec<String> = KNOWN_BADGES
        .iter()
        .copied()
        .filter(|badge| !scanned.contains(*badge))
        .map(String::from)
        .collect();


    evacuate_person.sort();
    missing_person.sort();

    Json(StatusResponse {
        total: evacuate_person.len() + missing_person.len(),
        evacuate_person,
        missing_person,
    })
}


async fn handle_status_summary(State(scanned): State<Arc<Mutex<HashSet<String>>>>)  -> Json<StatusResponseSummary> {
    let scanned = scanned.lock().unwrap();

    let mut evacuate_person: Vec<String> = scanned.iter().cloned().collect();
    let evacuate_person_count = evacuate_person.len();

    let mut missing_person: Vec<String> = KNOWN_BADGES
        .iter()
        .copied()
        .filter(|badge| !scanned.contains(*badge))
        .map(String::from)
        .collect();
    let missing_person_count = missing_person.len();


    evacuate_person.sort();
    missing_person.sort();

    Json(StatusResponseSummary {
        total: evacuate_person.len() + missing_person.len(),
        evacuate_person_count,
        missing_person_count,
    })
}
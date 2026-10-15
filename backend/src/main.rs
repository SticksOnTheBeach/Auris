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

use axum::{
    Router,
    extract::Json,
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::env;

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

    // build our application with a route
    let app = Router::new()
        .route("/api/scan", post(handle_scan));

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
async fn handle_scan(Json(payload): Json<ScanPayload>) -> Json<ScanResponse> {
    let status = if KNOWN_BADGES.contains(&payload.badge_hash.as_str()) {
        ScanStatus::Ok
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
/*
async fn handle_status(Json(payload): Json<ScanPayload>) -> Json<ScanResponse> {
    // Réponse HTTP
    Json(ScanResponse { status: ScanStatus::Ok })
*/

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
    extract::Json, // <-- Changement ici : Json vient de axum::extract, pas de la racine
    http::StatusCode,
    routing::post,
};
use serde::Deserialize;
use std::env;

// struct that just represented the data send at the /api/scan route
#[derive(Deserialize, Debug)]
struct ScanPayload {
    borne_id: String,
    badge_hash: String,
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
    let app = Router::new().route("/api/scan", post(handle_scan));

    // run our app with hyper, listening globally on port 3000
    //let listener = tokio::net::TcpListener::bind(&listening_address).await.unwrap();

    let listener = match tokio::net::TcpListener::bind(&listening_address).await {
        Ok(listener) => {
            println!("Server listening at http://{}/api/scan", listening_address);
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

// handler that just retrieves and processes the data
async fn handle_scan(Json(payload): Json<ScanPayload>) -> (StatusCode, &'static str) {
    // Ici, vous avez accès à vos données typées
    println!("Scan received at : {}", payload.borne_id);
    println!("Badges : {:?}", payload.badge_hash);

    // Réponse HTTP
    (StatusCode::OK, "Scan well received")
}

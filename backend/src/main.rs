use axum::{
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;

// On définit la structure des données que le C++ vas envoyer à /Api/post
// L'attribut `#[derive(Deserialize)]` permet à Axum de transformer le JSON reçu en cette structure.
#[derive(Deserialize, Debug)]
struct DonneesRecues {
    id: i32,
    message: String,
    valeur: f64,
}

fn main() {}

// 3. Le "handler" qui reçoit les données
// Axum extrait automatiquement le corps de la requête grâce à `Json(payload)`
async fn recevoir_infos_handler(Json(payload): Json<DonneesRecues>) -> &'static str {
    // Ici, vous avez accès aux variables extraites
    println!("Données reçues avec succès !");
    println!("   - ID : {}", payload.id);
    println!("   - Message : {}", payload.message);
    println!("   - Valeur : {}", payload.valeur);

    // On répond au programme C++ pour lui dire que tout s'est bien passé
    "Donnees bien recues par le serveur Rust !"
}

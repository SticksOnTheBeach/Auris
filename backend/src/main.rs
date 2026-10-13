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

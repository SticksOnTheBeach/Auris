# Dossier de Conception Architecturale : Système d'Évacuation "Secret Défense"

## 1. Contexte et Objectif
Dans le cadre d'un exercice d'évacuation (incendie, crise) pour une entreprise de haute sécurité, le décompte manuel aux points de rassemblement est inefficace. 
**Objectif :** Créer un système de pointage autonome, hermétique et ultra-rapide permettant de croiser les personnes entrées dans le bâtiment avec celles scannées au point de rassemblement, le tout en temps réel et sans aucune dépendance au Cloud (Air-Gapped).

## 2. Architecture Matérielle (Niveau Borne)
Le prototype utilise du matériel IoT industriel connecté en filaire (zéro ondes radio pour éviter le brouillage).
*   **Microcontrôleur :** WT32-ETH01 (ESP32 avec port Ethernet natif).
*   **Capteur RFID :** RC522 (Fréquence 13.56 MHz).
*   **Câblage (Bus SPI) :**
    *   `3.3V` ➔ `3V3`
    *   `GND` ➔ `GND`
    *   `RST` ➔ `IO 5`
    *   `SDA (SS)` ➔ `IO 14`
    *   `MOSI` ➔ `IO 15`
    *   `MISO` ➔ `IO 2`
    *   `SCK` ➔ `IO 12`

## 3. Architecture Réseau (Air-Gapped)
Le système repose sur un réseau local physiquement isolé (LAN privé) géré par un Switch Administrable.
*   **Protocole de transport :** TCP/IP (Garantit la livraison des paquets, contrairement à l'UDP).
*   **Plan d'adressage statique :**
    *   `10.0.0.10` : Serveur Central (Datacenter / Raspberry Pi).
    *   `10.0.0.20` : Borne Extérieure (WT32-ETH01).
    *   `10.0.0.30` : Tablette/PC de l'Agent de Sécurité.

## 4. Architecture Logicielle (Serveur 3-Tiers)
Le serveur physique (`10.0.0.10`) fait tourner 3 conteneurs Docker distincts pour isoler les services :
1.  **Docker Data (MySQL) :** Stocke la table des employés (avec le hash des badges) et les logs d'évacuation. N'est accessible que par le réseau virtuel Docker interne.
2.  **Docker Backend (Rust API) :** Le "cerveau". Développé avec `Axum` et `sqlx`. Écoute les requêtes TCP venant des bornes, vérifie les hashs en BDD et valide les évacuations.
3.  **Docker Frontend (Nginx/Qwik ou Client Lourd egui) :** Le serveur qui distribue l'interface graphique en temps réel (via WebSockets ou requêtes HTTP polling) à la tablette de l'agent.

---

## 5. Implémentation du Protocole TCP/HTTP (Code de communication)

Voici comment la borne et le serveur communiquent techniquement via TCP.

### A. Côté Borne (C++ sur le WT32-ETH01)
Le WT32 initialise sa puce Ethernet, ouvre un canal TCP (Client) vers le serveur, et forge une requête HTTP POST brute contenant l'empreinte du badge.

```cpp
#include <ETH.h>
#include <WiFiClient.h>

// IP du serveur Rust
const char* server_ip = "10.0.0.10";
const int server_port = 8080;

void sendBadgeScan(String badgeHash) {
    WiFiClient client; // Utilisé par ETH.h sous le capot pour le TCP
    
    // 1. Ouverture de la connexion TCP
    if (client.connect(server_ip, server_port)) {
        
        // Création du Payload JSON
        String jsonPayload = "{\"borne_id\":\"ext_nord\", \"badge_hash\":\"" + badgeHash + "\"}";
        
        // 2. Envoi de la requête HTTP POST sur le flux TCP
        client.println("POST /api/scan HTTP/1.1");
        client.println("Host: 10.0.0.10");
        client.println("Content-Type: application/json");
        client.print("Content-Length: ");
        client.println(jsonPayload.length());
        client.println(); // Ligne vide obligatoire dans le protocole HTTP
        client.println(jsonPayload); // Le corps de la requête
        
        Serial.println("Paquet TCP/HTTP envoyé avec succès.");
        client.stop(); // Fermeture propre de la connexion TCP
    } else {
        Serial.println("Erreur de connexion TCP au serveur.");
    }
}
```

### B. Côté Serveur (Rust avec Axum)
Le serveur ouvre un port réseau via le système d'exploitation, attend silencieusement qu'une connexion TCP s'établisse, et délègue le routage HTTP au framework Axum.

```rust
use axum::{routing::post, Router, Json};
use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct ScanPayload {
    borne_id: String,
    badge_hash: String,
}

#[tokio::main]
async fn main() {
    // 1. Définition de la route (L'oreille du serveur)
    let app = Router::new()
        .route("/api/scan", post(handle_scan));

    // 2. Ouverture de la socket d'écoute TCP sur le port 8080
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Backend Rust à l'écoute sur le port TCP 8080...");

    // 3. Démarrage du serveur qui boucle sur les connexions TCP entrantes
    axum::serve(listener, app).await.unwrap();
}

// Fonction asynchrone appelée à chaque requête valide
async fn handle_scan(Json(payload): Json<ScanPayload>) -> &'static str {
    println!("Scan reçu sur la borne {} ! Hash: {}", payload.borne_id, payload.badge_hash);
    
    // ... Logique sqlx pour interroger MySQL ici ...
    
    "ACK - Bien reçu" // Accusé de réception renvoyé à la borne
}
```
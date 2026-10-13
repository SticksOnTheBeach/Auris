# Auris : contexte du projet pour Claude Code

## Résumé

Auris est un système de pointage d'évacuation par badges RFID pour une entreprise de haute sécurité. Lors d'un exercice d'évacuation (incendie, crise), il remplace le décompte manuel au point de rassemblement : il croise les personnes entrées dans le bâtiment avec celles scannées au point de rassemblement, en temps réel, pour savoir qui manque.

Contraintes fortes :
- **Air-gapped** : réseau local privé physiquement isolé, aucune dépendance au Cloud, aucun CDN, aucune police ou icône chargée depuis internet.
- **Filaire uniquement** côté borne (Ethernet, zéro Wi-Fi) pour éviter le brouillage.
- **Rapide et fiable** : TCP/IP (livraison garantie), pas d'UDP.

## Architecture

| Composant | Techno | Où ça tourne |
|---|---|---|
| Borne extérieure | C++ (PlatformIO / Arduino ESP32) sur WT32-ETH01 + lecteur RC522 (13.56 MHz) | `10.0.0.20` |
| Backend API | Rust, Axum + sqlx | Serveur `10.0.0.10`, conteneur Docker |
| Base de données | MySQL | Serveur `10.0.0.10`, conteneur Docker, accessible uniquement via le réseau Docker interne |
| Frontend de l'agent | Qwik (TypeScript), build statique | Servi depuis le serveur, affiché sur la tablette/PC de l'agent `10.0.0.30` |

Réseau : LAN privé géré par un switch administrable, adressage statique.
- `10.0.0.10` : serveur central (Raspberry Pi ou PC)
- `10.0.0.20` : borne extérieure
- `10.0.0.30` : tablette/PC de l'agent de sécurité

### Répartition des responsabilités

- **Borne (C++)** : lit l'UID du badge, le hashe, envoie `POST /api/scan` en JSON, lit la réponse, renvoie en cas d'échec réseau, signale le résultat (LED/buzzer optionnel). Elle ne prend **aucune décision métier**.
- **Backend (Rust)** : toute l'intelligence. Vérifie le hash en base, détecte badge inconnu et badge déjà scanné, écrit les logs, calcule évacués/manquants, pousse les changements au front (WebSocket), filtre IP/MAC, vérifie le secret partagé des bornes.
- **Frontend (Qwik)** : affiche évacués, manquants et compteurs en temps réel, se reconnecte seul au WebSocket. Aucun calcul métier. Optionnel : pointage manuel par l'agent (badge oublié/perdu).
- **MySQL** : stocke employés (nom, hash du badge) et logs de scan. Aucune logique.

## Structure du dépôt (monorepo)

```
auris/
├── backend/              # Rust (Axum + sqlx)
│   ├── src/
│   ├── migrations/
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/             # Qwik
│   ├── src/
│   ├── package.json
│   └── Dockerfile        # ou conf Nginx
├── firmware/             # C++ du WT32-ETH01
│   ├── src/
│   └── platformio.ini
├── db/
│   └── init.sql          # schéma + faux employés de test
├── docs/
│   ├── dossier-architecture.md
│   └── api-contrat.md
├── scripts/
│   └── fake-scan.sh      # simule la borne avec curl
├── docker-compose.yml
├── .env.example
├── .gitignore
├── CLAUDE.md
└── README.md
```

Le firmware est dans son propre dossier (et non sous `backend/`) : il ne partage ni langage, ni outil de build, ni machine avec le serveur, et il se flashe sur le WT32, jamais dans Docker.

## Contrat d'API (à figer dans `docs/api-contrat.md`)

Requête de la borne :

```
POST /api/scan
Content-Type: application/json

{"borne_id": "ext_nord", "badge_hash": "<sha256 hex de l'UID>"}
```

Réponses prévues (proposition, à confirmer dans le contrat) :
- `ok` : badge connu, évacuation enregistrée
- `unknown` : badge inconnu en base
- `already_scanned` : badge déjà scanné pendant cet exercice

Autres routes prévues : `GET /api/status` (évacués, manquants, compteurs) et un WebSocket (`/ws`) qui diffuse chaque nouveau scan.

## État actuel

- **Pas encore de matériel** (ni WT32-ETH01, ni RC522, ni badges). Tout le développement se fait sur PC, en local, sur `localhost`.
- La borne est **simulée avec `curl`** (voir `scripts/fake-scan.sh`) : même format de requête, donc le vrai WT32 se branchera plus tard sans rien changer côté serveur.
- Le code C++ du dossier d'architecture est un prototype : il est à garder pour la réception du matériel, mais il doit être corrigé (voir ci-dessous).

### Simuler un scan

```bash
UID="04A1B2C3"
HASH=$(echo -n "$UID" | sha256sum | cut -d' ' -f1)

curl -X POST http://localhost:8080/api/scan \
  -H "Content-Type: application/json" \
  -d "{\"borne_id\":\"ext_nord\",\"badge_hash\":\"$HASH\"}"
```

Cas à tester : badge connu, badge inconnu, badge scanné deux fois.

## Ordre de développement

0. Figer le contrat d'API (`docs/api-contrat.md`).
1. MySQL : schéma (`employees`, `scans`, éventuellement `exercises`) + 5 à 10 faux employés ; `docker compose up db`.
2. Handler `POST /api/scan` en Rust (sqlx) avec les trois cas, testé avec `curl`.
3. État et temps réel : `GET /api/status`, puis WebSocket.
4. Frontend Qwik branché sur `/api/status` et le WebSocket.
5. `docker-compose.yml` complet.
6. Durcissement : filtrage IP/MAC, secret partagé borne/serveur, gestion des entrées dans le bâtiment.
7. Firmware : en dernier, car c'est le seul à dépendre du matériel.

## Points connus à traiter

- **Côté « entrées » manquant** : le dossier parle de croiser entrées et scans, mais rien n'enregistre qui est entré dans le bâtiment. Prévoir une borne d'entrée (même firmware, autre `borne_id` et autre IP) et une table/colonne pour ça.
- **Handler Rust incomplet** : le prototype se limite à un `println!` et un `"ACK"`. Reste à écrire la requête sqlx, « badge inconnu » et « badge déjà scanné ».
- **Firmware** : `client.stop()` est appelé juste après l'envoi sans lire la réponse. Lire la réponse (`ACK`) avant de fermer, et prévoir un renvoi en cas d'échec.
- **Broches SPI** : IO2, IO12 et IO15 sont des broches de *strapping* de l'ESP32 ; la borne peut ne pas démarrer selon ce que le RC522 impose à ces broches au boot. À tester dès réception du matériel.
- **Sécurité** : l'UID d'un badge 13.56 MHz se clone facilement et les bornes ne s'authentifient pas auprès du serveur. Ajouter au minimum un filtrage IP/MAC et un secret partagé dans la requête.

## Câblage RC522 ↔ WT32-ETH01 (SPI)

| RC522 | WT32-ETH01 |
|---|---|
| 3.3V | 3V3 |
| GND | GND |
| RST | IO 5 |
| SDA (SS) | IO 14 |
| MOSI | IO 15 |
| MISO | IO 2 |
| SCK | IO 12 |

## Conventions et règles de travail

- **Langue** : code, commentaires, commits et docs en français ; noms d'identifiants en anglais ou en français selon l'existant, mais cohérents.
- **Rust** : `cargo fmt` et `cargo clippy` avant de proposer du code ; pas de `unwrap()` dans le chemin des requêtes, gérer les erreurs proprement (retourner un code HTTP adapté).
- **Frontend** : aucune ressource externe (pas de CDN, pas de Google Fonts, pas d'icônes distantes) ; tout est embarqué dans le build. Build statique servi par Nginx (ou par Axum via `tower-http` `ServeDir`).
- **Docker** : MySQL n'est jamais exposé hors du réseau Docker interne.
- **Secrets** : ne jamais commiter de secret. Seul `.env.example` (valeurs factices) est versionné ; dans `firmware/`, IP et secret vont dans un `config.h` ignoré par Git, avec un `config.example.h` commité.
- **Données de test** : les hashs et noms dans `db/init.sql` sont fictifs. Jamais de vrais UID d'employés dans le dépôt.
- **`.gitignore`** : `backend/target/`, `frontend/node_modules/`, `frontend/dist/`, `firmware/.pio/`, `.env`, `firmware/**/config.h`.
- **Avant de modifier le contrat d'API**, mettre à jour `docs/api-contrat.md` et prévenir : borne, backend et front en dépendent tous les trois.
- Préférer des réponses concrètes et actionnables, et signaler directement les erreurs ou problèmes de conception plutôt que de les contourner.

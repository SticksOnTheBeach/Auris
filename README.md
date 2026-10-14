# Auris

Système de pointage d'évacuation par badges RFID pour une entreprise de haute sécurité.

Lors d'un exercice d'évacuation (incendie, crise), le décompte manuel au point de rassemblement est lent et source d'erreurs. Auris le remplace : il croise les personnes entrées dans le bâtiment avec celles scannées au point de rassemblement, en temps réel, pour savoir immédiatement **qui manque**.

## Principes

- **Air-gapped** : réseau local physiquement isolé, aucune dépendance au Cloud, aucune ressource externe (CDN, polices, icônes).
- **Filaire** : la borne communique en Ethernet, sans Wi-Fi, pour éviter tout brouillage.
- **Fiable** : transport TCP/IP, qui garantit la livraison des paquets.

## Architecture

```
Borne C++ ──POST /api/scan──► Backend Rust ──► MySQL
(WT32-ETH01 + RC522)               ▲
                                   │ GET /api/status
                            Frontend Qwik (tablette de l'agent)
```

| Composant | Techno | Adresse |
|---|---|---|
| Borne extérieure | C++, WT32-ETH01 + lecteur RC522 (13.56 MHz) | `10.0.0.20` |
| Backend API | Rust (Axum, sqlx) | `10.0.0.10`, conteneur Docker |
| Base de données | MySQL 8.4 | `10.0.0.10`, conteneur Docker (réseau interne uniquement) |
| Frontend | Qwik (TypeScript), build statique | servi par le serveur, affiché sur `10.0.0.30` |

La borne ne prend aucune décision : elle lit le badge, envoie son empreinte au backend et affiche la réponse. Toute la logique (badge connu, déjà scanné, calcul des manquants) est dans le backend.

## Structure du dépôt

```
auris/
├── backend/        # Rust (Axum + sqlx)
├── frontend/       # Qwik, interface de l'agent
├── firmware/       # C++ de la borne WT32-ETH01
├── db/             # schéma SQL et données de test
├── docs/           # dossier d'architecture, contrat d'API
├── scripts/        # outils (simulation de scan, etc.)
├── docker-compose.yml
├── .env.example
├── CLAUDE.md       # contexte du projet pour Claude Code
└── README.md
```

## API

| Route | Méthode | Rôle |
|---|---|---|
| `/api/scan` | POST | Enregistre le scan d'un badge envoyé par une borne |
| `/api/status` | GET | Liste des évacués et des manquants, avec le total |
| `/api/status/summary` | GET | Compteurs seuls : total, évacués, manquants |

Requête de la borne :

```json
{"borne_id": "ext_nord", "badge_hash": "<sha256 hex de l'UID>"}
```

Réponse de `/api/scan` :

```json
{"status": "ok"}
```

Valeurs possibles de `status` : `ok` (badge connu, évacuation enregistrée), `unknown` (badge inconnu), `already_scanned` (badge déjà scanné).

## Démarrage rapide (développement sur PC)

Aucun matériel n'est nécessaire : la borne est simulée avec `curl`.

### Prérequis

- Rust (stable) et `cargo`
- Docker et Docker Compose
- `curl` (et `jq`, optionnel, pour lire les réponses)

### Lancer la base de données

```bash
cp .env.example .env        # puis remplacer les mots de passe d'exemple
docker compose up -d db
docker compose ps           # attendre l'état "healthy"
```

### Lancer le backend

```bash
cd backend
cargo run
```

Le serveur écoute sur `http://localhost:8080`.

### Simuler une borne

```bash
UID="04A1B2C3"
HASH=$(echo -n "$UID" | sha256sum | cut -d' ' -f1)

curl -X POST http://localhost:8080/api/scan \
  -H "Content-Type: application/json" \
  -d "{\"borne_id\":\"ext_nord\",\"badge_hash\":\"$HASH\"}"

curl http://localhost:8080/api/status
```

Cas à essayer : badge connu, badge inconnu, même badge scanné deux fois.

## Câblage de la borne (RC522 ↔ WT32-ETH01, bus SPI)

| RC522 | WT32-ETH01 |
|---|---|
| 3.3V | 3V3 |
| GND | GND |
| RST | IO 5 |
| SDA (SS) | IO 14 |
| MOSI | IO 15 |
| MISO | IO 2 |
| SCK | IO 12 |

IO2, IO12 et IO15 sont des broches de *strapping* de l'ESP32 : à vérifier dès la réception du matériel, la borne pouvant ne pas démarrer selon l'état imposé par le RC522.

## État d'avancement

- [x] Backend : `POST /api/scan` (trois cas, état en mémoire)
- [x] Backend : `GET /api/status` et `GET /api/status/summary`
- [ ] Base MySQL : schéma, employés de test, `docker-compose` *(en cours)*
- [ ] Backend : remplacement de l'état en mémoire par sqlx
- [ ] Backend : WebSocket pour le temps réel
- [ ] Frontend Qwik
- [ ] Sécurité : filtrage IP/MAC, secret partagé borne/serveur
- [ ] Gestion des entrées dans le bâtiment (borne d'entrée)
- [ ] Firmware de la borne (nécessite le matériel)

## Workflow Git

`main` contient toujours ce qui fonctionne. Chaque évolution passe par une branche courte, fusionnée dès qu'elle est testée puis supprimée.

- Préfixes : `feat/`, `refacto/`, `fix/`, `docs/`
- Format des commits : `type::zone::détail::techno : message`
- Exception : `feat/firmware-rc522` reste ouverte jusqu'au test sur le vrai matériel

## Sécurité

- Ne jamais commiter de `.env` 
- Les hashs et noms de `db/init.sql` sont fictifs ; aucun vrai identifiant d'employé n'a sa place dans le dépôt.
- En production, MySQL n'est exposé qu'au réseau Docker interne.
- Limite connue : l'UID d'un badge 13.56 MHz se clone facilement. Le filtrage IP/MAC et le secret partagé des bornes sont prévus pour limiter ce risque.

## Documentation

- [`docs/dossier-architecture.md`](docs/dossier-architecture.md) : conception complète du système
- [`docs/api-contrat.md`](docs/api-contrat.md) : contrat d'API entre la borne, le backend et le frontend
- [`CLAUDE.md`](CLAUDE.md) : contexte du projet pour Claude Code
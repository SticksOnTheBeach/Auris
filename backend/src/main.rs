use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};

// ---------- Request / response formats ----------

// What the scanner sends
#[derive(Deserialize)]
struct ScanRequest {
    borne_id: String,
    badge_hash: String,
}

// What the server answers to the scanner
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
struct StatusSummaryResponse {
    total: usize,
    evacuate_person: usize,
    missing_person: usize,
}

// ---------- Errors ----------

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

type ApiError = (StatusCode, Json<ErrorBody>);

// The details stay in the server terminal, the scanner only receives a code
fn storage_error(error: sqlx::Error) -> ApiError {
    eprintln!("Error: {}", error);
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorBody {
            error: "storage_unavailable",
        }),
    )
}

// ---------- Handlers ----------

// POST /api/scan
async fn handle_scan(
    State(pool): State<PgPool>,
    Json(request): Json<ScanRequest>,
) -> Result<Json<ScanResponse>, ApiError> {
    println!("scan: borne={} badge={}", request.borne_id, request.badge_hash);

    // A single INSERT: it only succeeds if the badge belongs to a known employee
    // AND has no "ok" scan yet (thanks to the unique index in init.sql).
    let inserted = sqlx::query(
        "INSERT INTO scans (employee_id, badge_hash, borne_id, result)
         SELECT id, badge_hash, $1::text, 'ok' FROM employees WHERE badge_hash = $2
         ON CONFLICT (badge_hash) WHERE result = 'ok' DO NOTHING",
    )
        .bind(&request.borne_id)
        .bind(&request.badge_hash)
        .execute(&pool)
        .await
        .map_err(storage_error)?
        .rows_affected();

    let status = if inserted == 1 {
        ScanStatus::Ok
    } else {
        // Nothing inserted: either the badge is unknown, or it was already scanned
        let known = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM employees WHERE badge_hash = $1)",
        )
            .bind(&request.badge_hash)
            .fetch_one(&pool)
            .await
            .map_err(storage_error)?;

        if known {
            ScanStatus::AlreadyScanned
        } else {
            ScanStatus::Unknown
        }
    };

    Ok(Json(ScanResponse { status }))
}

// GET /api/status
async fn handle_status(State(pool): State<PgPool>) -> Result<Json<StatusResponse>, ApiError> {
    let evacuated = sqlx::query_scalar::<_, String>(
        "SELECT e.badge_hash FROM employees e
         WHERE EXISTS (SELECT 1 FROM scans s WHERE s.badge_hash = e.badge_hash AND s.result = 'ok')
         ORDER BY e.badge_hash",
    )
        .fetch_all(&pool)
        .await
        .map_err(storage_error)?;

    let missing = sqlx::query_scalar::<_, String>(
        "SELECT e.badge_hash FROM employees e
         WHERE NOT EXISTS (SELECT 1 FROM scans s WHERE s.badge_hash = e.badge_hash AND s.result = 'ok')
         ORDER BY e.badge_hash",
    )
        .fetch_all(&pool)
        .await
        .map_err(storage_error)?;

    Ok(Json(StatusResponse {
        total: evacuated.len() + missing.len(),
        evacuate_person: evacuated,
        missing_person: missing,
    }))
}

// GET /api/status/summary
async fn handle_summary(
    State(pool): State<PgPool>,
) -> Result<Json<StatusSummaryResponse>, ApiError> {
    let (total, evacuated): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM scans s WHERE s.badge_hash = e.badge_hash AND s.result = 'ok'))
         FROM employees e",
    )
        .fetch_one(&pool)
        .await
        .map_err(storage_error)?;

    let total = total as usize;
    let evacuated = evacuated as usize;

    Ok(Json(StatusSummaryResponse {
        total,
        evacuate_person: evacuated,
        // saturating_sub: never panics, even if scans somehow exceed the headcount
        missing_person: total.saturating_sub(evacuated),
    }))
}

// ---------- Startup ----------

#[tokio::main]
async fn main() {
    // Load the .env file (looked up in the current directory, then its parents)
    dotenvy::dotenv().ok();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Error: missing DATABASE_URL environment variable");
            std::process::exit(1);
        }
    };

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("Error: failed to connect to the database ({})", error);
            std::process::exit(1);
        }
    };

    let app = Router::new()
        .route("/api/scan", post(handle_scan))
        .route("/api/status", get(handle_status))
        .route("/api/status/summary", get(handle_summary))
        .with_state(pool);

    let address = SocketAddr::from(([0, 0, 0, 0], 8080));

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => {
            println!("Server listening at http://{}/api/scan", address);
            listener
        }
        Err(error) => {
            eprintln!("Error: failed to bind {} ({})", address, error);
            std::process::exit(1);
        }
    };

    if let Err(error) = axum::serve(listener, app).await {
        eprintln!("Error: server stopped ({})", error);
        std::process::exit(1);
    }
}
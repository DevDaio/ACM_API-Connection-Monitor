// ─── Modul-Deklarationen ───
// Importiert die Untermodule: service_modules (Business-Logik), types (Datenstrukturen), handlers (HTTP-Handler)
mod service_modules;
mod types;
mod handlers;

use axum::{
    http::Method,
    routing::{delete, get, post, put},
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use sqlx::postgres::PgPoolOptions;

use crate::types::AppState;

// ════════════════════════════════════════════════════════════════
//  Einstiegspunkt: Server-Konfiguration & Start
// ════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Lädt .env-Datei (für DATABASE_URL, BACKEND_HOST, BACKEND_PORT)
    dotenvy::dotenv().ok();

    // DB-Verbindung: URL aus Umgebungsvariable oder Fallback
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:admin123!@localhost:5432/database-acm".to_string());

    // Verbindungspool mit max. 5 gleichzeitigen Verbindungen
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database");

    // ─── Tabellen anlegen (CREATE TABLE IF NOT EXISTS) ───
    // "user":    Benutzerkonten mit E-Mail und bcrypt-gehashtem Passwort
    // "endpoint":   Überwachte URLs
    // "userendpoint": Viele-zu-Viele-Verknüpfung User <-> Endpoint (mit CASCADE DELETE)
    // "intervall":  Check-Intervalle pro Endpoint (in Sekunden)
    // "log":       Statusaufzeichnungen mit Datum, Uhrzeit und URL
    sqlx::query(r#"CREATE TABLE IF NOT EXISTS "user" (userid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY, emailadress VARCHAR(100) NOT NULL UNIQUE, password VARCHAR(100) NOT NULL)"#).execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS endpoint (endpointid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY, url VARCHAR(300) NOT NULL, check_type VARCHAR(10) NOT NULL DEFAULT 'http')").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS userendpoint (userid INTEGER NOT NULL, endpointid INTEGER NOT NULL, PRIMARY KEY (userid, endpointid), FOREIGN KEY (userid) REFERENCES \"user\"(userid) ON DELETE CASCADE, FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS intervall (endpointid INTEGER PRIMARY KEY, seconds INTEGER NOT NULL, FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS log (endpointid INTEGER NOT NULL, status BOOLEAN, statusdate DATE NOT NULL DEFAULT CURRENT_DATE, statustime TIME NOT NULL DEFAULT CURRENT_TIME, url VARCHAR(300), check_type VARCHAR(10), FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE)").execute(&pool).await?;
    // Migration: url-Spalte nachträglich hinzugefügt (für bestehende DBs)
    sqlx::query("ALTER TABLE log ADD COLUMN IF NOT EXISTS url VARCHAR(300)").execute(&pool).await?;
    // Migration: status-Spalte auf NULL erlaubt (für URL-Edit-Events ohne Status)
    sqlx::query("ALTER TABLE log ALTER COLUMN status DROP NOT NULL").execute(&pool).await?;
    // Migration (nur für bestehende DBs): check_type für ICMP/HTTP/TCP-Auswahl
    sqlx::query("ALTER TABLE endpoint ADD COLUMN IF NOT EXISTS check_type VARCHAR(10) NOT NULL DEFAULT 'http'").execute(&pool).await?;
    // Migration (nur für bestehende DBs): check_type in Log-Tabelle
    sqlx::query("ALTER TABLE log ADD COLUMN IF NOT EXISTS check_type VARCHAR(10)").execute(&pool).await?;
    println!("Tables ready");

    // ─── Hintergrund-Monitoring-Loop starten ───
    // Läuft in einem separaten Tokio-Task und prüft Endpunkte (HTTP/TCP/ICMP) im konfigurierten Intervall
    let monitor_pool = pool.clone();
    tokio::spawn(async move {
        crate::service_modules::async_services::run_monitoring_loop(monitor_pool).await;
    });
    println!("Monitoring loop started");

    // ─── AppState (Shared State) für Axum ───
    // Arc = Thread-sicherer Referenzzähler – alle Handler teilen sich denselben Pool
    // sessions = In-Memory-Session-Speicher (token → userid)
    let state = Arc::new(AppState {
        pool,
        sessions: RwLock::new(HashMap::new()),
    });

    // ─── CORS-Konfiguration ───
    // Erlaubt Anfragen von beliebigen Origins (Any) – für Entwicklung/Frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // ─── Routen ───
    // Alle Pfade beginnen mit /acm
    let app = Router::new()
        .route("/acm", get(handlers::handle_healthcheck))
        .route("/acm/login", post(handlers::handle_login))
        .route("/acm/createAccount", post(handlers::handle_create_account))
        .route("/acm/home", get(handlers::handle_home))
        .route("/acm/user", get(handlers::handle_user))
        .route("/acm/user/changePassword", put(handlers::handle_change_password))
        .route("/acm/user/changeEmail", put(handlers::handle_change_email))
        .route("/acm/user/deleteAccount", delete(handlers::handle_delete_account))
        .route("/acm/addEndpoint", put(handlers::handle_add_endpoint))
        .route("/acm/setIntervall", put(handlers::handle_set_intervall))
        .route("/acm/deleteConfirm", put(handlers::handle_delete_endpoint))
        .route("/acm/updateEndpoint", put(handlers::handle_update_endpoint))
        .route("/acm/log", get(handlers::handle_log))
        .layer(cors)
        .with_state(state);

    // ─── Server binden und starten ───
    // BACKEND_HOST / BACKEND_PORT aus Umgebungsvariablen (Default: 0.0.0.0:3000)
    let bind_host = std::env::var("BACKEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_port = std::env::var("BACKEND_PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("{}:{}", bind_host, bind_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("Server running on http://{}", bind_addr);
    // axum::serve startet den HTTP-Server (blockiert, bis das Programm beendet wird)
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

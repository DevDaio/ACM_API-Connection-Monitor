// ─── Modul-Deklaration ───
// Deklariert das Modul service_modules (async_services), das in service_modules/ definiert ist
mod service_modules;

// ─── Externe Crates ───
// axum für HTTP-Routing, serde für Serialisierung/Deserialisierung,
// sqlx für DB-Zugriff, tower_http für CORS
use axum::{
    extract::Query,
    http::Method,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use service_modules::async_services;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

// ─── App-Zustand ───
// Globaler App-Zustand: hält den PostgreSQL-Verbindungspool
// Arc ermoeglicht Thread-sicheres Teilen zwischen allen Handlern
struct AppState {
    pool: PgPool,
}

// ════════════════════════════════════════════════════════════════
//  Request-Typen (Deserialize) – werden aus JSON geparst
// ════════════════════════════════════════════════════════════════

// Request-Body für die Account-Erstellung: E-Mail und Passwort
#[derive(Deserialize)]
struct CreateAccountReq {
    email: String,
    password: String,
}

// Request-Body für den Login: E-Mail und Passwort
#[derive(Deserialize)]
struct LoginReq {
    email: String,
    password: String,
}

// Request-Body für Passwortänderung: User-ID, altes und neues Passwort
#[derive(Deserialize)]
struct ChangePasswordReq {
    userid: i32,
    old_password: String,
    new_password: String,
}

// Request-Body für E-Mail-Änderung: User-ID und neue E-Mail
#[derive(Deserialize)]
struct ChangeEmailReq {
    userid: i32,
    new_email: String,
}

// Request-Body für Account-Löschung: nur User-ID
#[derive(Deserialize)]
struct DeleteAccountReq {
    userid: i32,
}

// Request-Body zum Hinzufügen eines Endpunkts: User-ID und URL
#[derive(Deserialize)]
struct AddEndpointReq {
    userid: i32,
    url: String,
}

// Request-Body zum Setzen des Monitoring-Intervalls: Endpunkt-ID und Sekunden
#[derive(Deserialize)]
struct SetIntervallReq {
    endpointid: i32,
    seconds: i32,
}

// Request-Body zum Löschen eines Endpunkts: nur Endpunkt-ID
#[derive(Deserialize)]
struct DeleteEndpointReq {
    endpointid: i32,
}

// Request-Body zum Aktualisieren eines Endpunkts: Endpunkt-ID und neue URL
#[derive(Deserialize)]
struct UpdateEndpointReq {
    endpointid: i32,
    url: String,
}

// Query-Parameter für einfache ID-basierte GET-Endpunkte (z. B. /acm/home?id=1)
#[derive(Deserialize)]
struct IdParam {
    id: i32,
}

// ════════════════════════════════════════════════════════════════
//  Response-Typen (Serialize) – werden zu JSON serialisiert
// ════════════════════════════════════════════════════════════════

// Response für Login/Account-Erstellung: User-ID und E-Mail-Adresse
#[derive(Serialize)]
struct LoginRes {
    userid: i32,
    emailadress: String,
}

// Einheitliche Fehler-Response mit Fehlermeldung
#[derive(Serialize)]
struct ErrorRes {
    error: String,
}

// ════════════════════════════════════════════════════════════════
//  Route-Handler – jeder Handler ist eine async fn
// ════════════════════════════════════════════════════════════════

// [GET /acm] Gesundheitscheck – gibt Status "ok" zurück
// Dient als Lebenszeichen für Load-Balancer oder externe Checks
async fn handle_healthcheck() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "message": "ACM API Connection Monitor" }))
}

// [POST /acm/createAccount] Erstellt einen neuen Account.
// 1. Prüft, ob die E-Mail bereits existiert (sonst 409 CONFLICT)
// 2. Hasht das Passwort mit bcrypt (Default-Cost = 12)
// 3. Speichert user in der DB und gibt userid + email zurück
async fn handle_create_account(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<CreateAccountReq>,
) -> Result<Json<LoginRes>, (axum::http::StatusCode, Json<ErrorRes>)> {
    // Prüfen, ob E-Mail schon vergeben ist
    let existing = async_services::get_user_by_email(&state.pool, &body.email)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    if existing.is_some() {
        return Err((
            axum::http::StatusCode::CONFLICT,
            Json(ErrorRes {
                error: "Email already exists".to_string(),
            }),
        ));
    }

    // Passwort hashen – bcrypt mit Default-Cost
    // DEFAULT_COST = 12 => ~250ms Hash-Zeit (Sicherheit vs. Performance)
    let hashed = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    // User in DB anlegen und zurückgeben
    let user = async_services::create_account(&state.pool, &body.email, &hashed)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    Ok(Json(LoginRes {
        userid: user.userid,
        emailadress: user.emailadress,
    }))
}

// [POST /acm/login] Authentifiziert einen Benutzer.
// Validiert E-Mail + Passwort gegen die DB.
// Bei Erfolg: userid + email. Bei Fehler: 401 UNAUTHORIZED.
async fn handle_login(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<LoginReq>,
) -> Result<Json<LoginRes>, (axum::http::StatusCode, Json<ErrorRes>)> {
    // User anhand der E-Mail laden
    let user = async_services::get_user_by_email(&state.pool, &body.email)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?
        // Wenn keine E-Mail gefunden -> 401 (gleiche Fehlermeldung wie bei falschem PW,
        // damit ein Angreifer nicht weiss, ob die E-Mail existiert)
        .ok_or((
            axum::http::StatusCode::UNAUTHORIZED,
            Json(ErrorRes {
                error: "Invalid email or password".to_string(),
            }),
        ))?;

    // bcrypt-Vergleich: gegebene Passwort vs. gehashter String aus der DB
    let valid = bcrypt::verify(&body.password, &user.password).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    if !valid {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            Json(ErrorRes {
                error: "Invalid email or password".to_string(),
            }),
        ));
    }

    Ok(Json(LoginRes {
        userid: user.userid,
        emailadress: user.emailadress,
    }))
}

// [GET /acm/home?id=...] Liefert alle Endpunkte eines Benutzers.
// Der Query-Parameter ?id= wird via "Query<IdParam>" extrahiert.
// Gibt ein Array von EndpointExtended zurück (enthält Status, Interval, History).
async fn handle_home(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Query(params): Query<IdParam>,
) -> Result<Json<Vec<async_services::EndpointExtended>>, (axum::http::StatusCode, Json<ErrorRes>)> {
    let endpoints = async_services::get_user_endpoints(&state.pool, params.id)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(endpoints))
}

// [GET /acm/user?id=...] Gibt die Daten eines Benutzers zurück
async fn handle_user(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Query(params): Query<IdParam>,
) -> Result<Json<async_services::User>, (axum::http::StatusCode, Json<ErrorRes>)> {
    let user = async_services::get_user_by_id(&state.pool, params.id)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(user))
}

// [PUT /acm/user/changePassword] Ändert das Passwort.
// 1. User aus DB laden 2. Altes Passwort validieren 3. Neues hashen 4. Speichern
async fn handle_change_password(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<ChangePasswordReq>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ErrorRes>)> {
    let user = async_services::get_user_by_id(&state.pool, body.userid)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    // Altes Passwort verifizieren, bevor das neue gesetzt wird
    let valid = bcrypt::verify(&body.old_password, &user.password).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    if !valid {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            Json(ErrorRes {
                error: "Current password is incorrect".to_string(),
            }),
        ));
    }

    // Neues Passwort hashen und speichern
    let hashed = bcrypt::hash(&body.new_password, bcrypt::DEFAULT_COST).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    async_services::change_password(&state.pool, body.userid, &hashed)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// [PUT /acm/user/changeEmail] Ändert die E-Mail-Adresse eines Benutzers
async fn handle_change_email(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<ChangeEmailReq>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ErrorRes>)> {
    async_services::change_email(&state.pool, body.userid, &body.new_email)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// [DELETE /acm/user/deleteAccount] Löscht einen Account
// CASCADE in der DB löscht automatisch zugehörige Endpunkte, Logs, Intervalle
async fn handle_delete_account(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<DeleteAccountReq>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ErrorRes>)> {
    async_services::delete_account(&state.pool, body.userid)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// [PUT /acm/addEndpoint] Fügt einen neuen Endpunkt für einen Benutzer hinzu.
// Erzeugt einen Eintrag in endpoint + userendpoint. Gibt die neue endpointid zurück.
async fn handle_add_endpoint(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<AddEndpointReq>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ErrorRes>)> {
    let endpointid = async_services::add_endpoint(&state.pool, body.userid, &body.url)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "endpointid": endpointid })))
}

// [PUT /acm/setIntervall] Setzt das Monitoring-Intervall für einen Endpunkt.
// Nutzt ON CONFLICT (upsert) – wenn bereits ein Intervall existiert, wird es überschrieben.
async fn handle_set_intervall(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<SetIntervallReq>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ErrorRes>)> {
    async_services::set_intervall(&state.pool, body.endpointid, body.seconds)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// [PUT /acm/deleteConfirm] Löscht einen Endpunkt.
// Entfernt zugehörige Logs, Intervall, userendpoint-Verknüpfung und endpoint selbst.
async fn handle_delete_endpoint(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<DeleteEndpointReq>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ErrorRes>)> {
    async_services::delete_endpoint(&state.pool, body.endpointid)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// [GET /acm/log?id=...] Liefert die Monitoring-Logs eines Endpunkts.
// Sortiert absteigend nach Datum + Uhrzeit (neueste zuerst).
async fn handle_log(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Query(params): Query<IdParam>,
) -> Result<Json<Vec<async_services::Log>>, (axum::http::StatusCode, Json<ErrorRes>)> {
    let logs = async_services::get_log(&state.pool, params.id)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(logs))
}

// [PUT /acm/updateEndpoint] Aktualisiert die URL eines Endpunkts
// Schreibt einen Log-Eintrag mit der neuen URL, damit Änderungen nachvollziehbar sind
async fn handle_update_endpoint(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<UpdateEndpointReq>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ErrorRes>)> {
    async_services::update_endpoint(&state.pool, body.endpointid, &body.url)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    async_services::insert_log(
        &state.pool,
        body.endpointid,
        None,
        Some(&body.url),
    )
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ════════════════════════════════════════════════════════════════
//  Main – Einstiegspunkt
// ════════════════════════════════════════════════════════════════

// #[tokio::main] ist das Makro, das die async-Runtime initialisiert
// Es wrappt main() in einen tokio::runtime::Runtime
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Lädt .env-Datei (nicht kritisch – unwrap_or_else gibt Default-String)
    dotenv::dotenv().ok();

    // DATABASE_URL aus Umgebungsvariable lesen, sonst Default für lokale Entwicklung
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin@localhost:5432/mydb".to_string());

    // PgPoolOptions: Connection-Pool mit max 5 gleichzeitigen Verbindungen
    // sqlx führt das Verbinden erst bei .connect() aus – pooled connections sind lazy
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database");

    // ─── Tabellen automatisch anlegen (falls nicht vorhanden) ───
    sqlx::query(r#"CREATE TABLE IF NOT EXISTS "user" (userid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY, emailadress VARCHAR(100) NOT NULL UNIQUE, password VARCHAR(100) NOT NULL)"#).execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS endpoint (endpointid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY, url VARCHAR(300) NOT NULL)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS userendpoint (userid INTEGER NOT NULL, endpointid INTEGER NOT NULL, PRIMARY KEY (userid, endpointid), FOREIGN KEY (userid) REFERENCES \"user\"(userid) ON DELETE CASCADE, FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS intervall (endpointid INTEGER PRIMARY KEY, seconds INTEGER NOT NULL, FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS log (endpointid INTEGER NOT NULL, status BOOLEAN, statusdate DATE NOT NULL DEFAULT CURRENT_DATE, statustime TIME NOT NULL DEFAULT CURRENT_TIME, url VARCHAR(300), FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE)").execute(&pool).await?;
    sqlx::query("ALTER TABLE log ADD COLUMN IF NOT EXISTS url VARCHAR(300)").execute(&pool).await?;
    sqlx::query("ALTER TABLE log ALTER COLUMN status DROP NOT NULL").execute(&pool).await?;
    println!("Tables ready");

    // ─── Monitoring-Loop in eigenem Task starten ───
    // tokio::spawn startet einen grünen Thread (Task) im Hintergrund.
    // Der Loop läuft unabhängig vom HTTP-Server und überwacht Endpunkte.
    let monitor_pool = pool.clone();
    tokio::spawn(async move {
        async_services::run_monitoring_loop(monitor_pool).await;
    });
    println!("Monitoring loop started");

    // AppState in Arc verpacken – geteilter Zustand für alle Handler
    let state = Arc::new(AppState { pool });

    // ─── CORS-Konfiguration ───
    // Erlaubt alle Origins und alle Header – für Entwicklung okay,
    // in Produktion sollte allow_origin auf die konkrete Frontend-Domain gesetzt werden
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // ─── Router bauen ───
    // Jede Route wird mit HTTP-Methode + Path an den Handler gebunden
    let app = Router::new()
        .route("/acm", get(handle_healthcheck))
        .route("/acm/login", post(handle_login))
        .route("/acm/createAccount", post(handle_create_account))
        .route("/acm/home", get(handle_home))
        .route("/acm/user", get(handle_user))
        .route("/acm/user/changePassword", put(handle_change_password))
        .route("/acm/user/changeEmail", put(handle_change_email))
        .route("/acm/user/deleteAccount", delete(handle_delete_account))
        .route("/acm/addEndpoint", put(handle_add_endpoint))
        .route("/acm/setIntervall", put(handle_set_intervall))
        .route("/acm/deleteConfirm", put(handle_delete_endpoint))
        .route("/acm/updateEndpoint", put(handle_update_endpoint))
        .route("/acm/log", get(handle_log))
        .layer(cors)
        .with_state(state);

    // ─── HTTP-Server starten ───
    // BACKEND_HOST + BACKEND_PORT aus .env oder Default (0.0.0.0:3000)
    let bind_host = std::env::var("BACKEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_port = std::env::var("BACKEND_PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("{}:{}", bind_host, bind_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("Server running on http://{}", bind_addr);
    // axum::serve ist der async-HTTP-Server – blockt bis zum Abbruch
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// Import muss hier unten stehen (use-Anweisungen muessen in Rust nicht oben sein,
// aber Konvention ist, sie an den Dateianfang zu setzen – dieser hier ist verschoben,
// weil PgPoolOptions erst nach sqlx importiert wird)
use sqlx::postgres::PgPoolOptions;

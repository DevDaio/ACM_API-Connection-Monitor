/* ═══════════════════════════════════════════════════════
 * 📦 Server-Einstiegspunkt + Router + Handler
 *
 * 🎯 ZWECK:
 * Startet den HTTP-Server, definiert alle API-Routen
 * und verbindet Handler mit Datenbank-Queries.
 *
 * 📥 INPUT:
 * - DATABASE_URL aus Umgebungsvariable oder Default
 *
 * 📤 OUTPUT:
 * - HTTP-Server auf Port 3000
 * - 13 API-Endpoints unter /acm/*
 *
 * 🔗 DEPENDENCIES:
 * - axum: Routing + Handler
 * - tokio: Async-Runtime
 * - serde/serde_json: JSON (De)Serialization
 * - tower-http: CORS
 * - bcrypt: Passwort-Hashing
 * - service_modules::async_services: DB-Queries
 * - dotenv: .env-Datei laden
 *
 * 💡 KONZEPTE:
 * - Shared State mit Arc<AppState>
 * - axum Extractor (State, Query, Json)
 * - CORS Middleware
 * - tokio::spawn für Background-Tasks
 *
 * ⚠️ WICHTIG ZU WISSEN:
 * - CORS erlaubt EVERY Origin (nur für Dev!)
 * - Fehler werden als (StatusCode, Json<ErrorRes>) getypt
 *
 * 🎓 LERN-TIPP:
 * Lies erst die Router-Definition (main()), dann
 * einzelne Handler. Jeder Handler folgt dem selben
 * Pattern: State + Parameter → async_services → Response
 * ═══════════════════════════════════════════════════════ */

/* ═══════════════════════════════════════════════════════
 * 📦 MODUL-DEKORATION
 * Macht das service_modules-Verzeichnis verfügbar.
 * ═══════════════════════════════════════════════════════ */
mod service_modules;

/* ═══════════════════════════════════════════════════════
 * 📦 IMPORTS
 * axum: Router, Extractor, Handler-Factory
 * serde: Request/Response-Typen
 * service_modules::async_services: DB-Zugriff
 * sqlx::PgPool: Datenbank-Pool
 * std::sync::Arc: Thread-sicherer Shared State
 * tower_http::cors: CORS-Header
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 APP-STATE
 * Ein Struct, das den globalen Anwendungszustand hält.
 * Hier: Nur der PostgreSQL-Connection-Pool.
 * Wird per Arc<AppState> an alle Handler verteilt.
 * ═══════════════════════════════════════════════════════ */
struct AppState {
    pool: PgPool,
}

/* ═══════════════════════════════════════════════════════
 * 📦 REQUEST-TYPEN
 * Jeder API-Endpoint bekommt einen eigenen Deserialize-
 * Struct. Die Feldnamen entsprechen den JSON-Keys.
 * ═══════════════════════════════════════════════════════ */
#[derive(Deserialize)]
struct CreateAccountReq {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginReq {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct ChangePasswordReq {
    userid: i32,
    old_password: String,
    new_password: String,
}

#[derive(Deserialize)]
struct ChangeEmailReq {
    userid: i32,
    new_email: String,
}

#[derive(Deserialize)]
struct DeleteAccountReq {
    userid: i32,
}

#[derive(Deserialize)]
struct AddEndpointReq {
    userid: i32,
    url: String,
}

#[derive(Deserialize)]
struct SetIntervallReq {
    endpointid: i32,
    seconds: i32,
}

#[derive(Deserialize)]
struct DeleteEndpointReq {
    endpointid: i32,
}

#[derive(Deserialize)]
struct UpdateEndpointReq {
    endpointid: i32,
    url: String,
}

/* ═══════════════════════════════════════════════════════
 * 📦 QUERY-PARAMETER
 * Für GET-Endpoints, die ?id=N erwarten.
 * ═══════════════════════════════════════════════════════ */
#[derive(Deserialize)]
struct IdParam {
    id: i32,
}

/* ═══════════════════════════════════════════════════════
 * 📦 RESPONSE-TYPEN
 * LoginRes: Erfolgreicher Login/Registrierung
 * ErrorRes: Einheitliches Error-Format
 * ═══════════════════════════════════════════════════════ */
#[derive(Serialize)]
struct LoginRes {
    userid: i32,
    emailadress: String,
}

#[derive(Serialize)]
struct ErrorRes {
    error: String,
}

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: HEALTHCHECK
 * GET /acm → {"status": "ok", "message": "..."}
 * Einfachster Handler: kein State, kein Parameter.
 * ═══════════════════════════════════════════════════════ */
async fn handle_healthcheck() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "message": "ACM API Connection Monitor" }))
}

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: CREATE ACCOUNT
 * POST /acm/createAccount
 * 1. Prüfen, ob Email bereits existiert
 * 2. Passwort mit bcrypt hashen
 * 3. User in DB anlegen
 * 4. userid + email zurückgeben
 *
 * 💡 LERN-TIPP: Pattern "prüfen → verarbeiten → antworten"
 * ═══════════════════════════════════════════════════════ */
async fn handle_create_account(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<CreateAccountReq>,
) -> Result<Json<LoginRes>, (axum::http::StatusCode, Json<ErrorRes>)> {
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

    let hashed = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: LOGIN
 * POST /acm/login
 * 1. User per Email finden
 * 2. Passwort mit bcrypt verifizieren
 * 3. userid + email zurückgeben
 *
 * ⚠️ WICHTIG: Gleiche Fehlermeldung bei "User nicht
 * gefunden" und "Passwort falsch" – verhindert
 * Enumeration gültiger Emails.
 * ═══════════════════════════════════════════════════════ */
async fn handle_login(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<LoginReq>,
) -> Result<Json<LoginRes>, (axum::http::StatusCode, Json<ErrorRes>)> {
    let user = async_services::get_user_by_email(&state.pool, &body.email)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?
        .ok_or((
            axum::http::StatusCode::UNAUTHORIZED,
            Json(ErrorRes {
                error: "Invalid email or password".to_string(),
            }),
        ))?;

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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: HOME (Endpoint-Liste)
 * GET /acm/home?id=N
 * Gibt alle Endpoints eines Users mit aktuellem Status,
 * Uptime-Dauer, Intervall und Status-Historie zurück.
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: USER (Daten abrufen)
 * GET /acm/user?id=N
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: CHANGE PASSWORD
 * PUT /acm/user/changePassword
 * Alt → Neu: Verifiziert altes Passwort, hasht neues.
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: CHANGE EMAIL
 * PUT /acm/user/changeEmail
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: DELETE ACCOUNT
 * DELETE /acm/user/deleteAccount
 * Löscht User + alle verknüpften Daten (CASCADE).
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: ADD ENDPOINT
 * PUT /acm/addEndpoint
 * Legt neuen Endpoint an, verknüpft ihn mit User.
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: SET INTERVALL
 * PUT /acm/setIntervall
 * Setzt/aktualisiert Prüfintervall (Upsert).
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: DELETE ENDPOINT
 * PUT /acm/deleteConfirm
 * Löscht Endpoint + zugehörige Logs/Intervalle (manuell).
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: LOG
 * GET /acm/log?id=N
 * Gibt alle Log-Einträge eines Endpoints zurück.
 * ═══════════════════════════════════════════════════════ */
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

/* ═══════════════════════════════════════════════════════
 * 📦 HANDLER: UPDATE ENDPOINT (URL ändern)
 * PUT /acm/updateEndpoint
 * ═══════════════════════════════════════════════════════ */
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
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/* ═══════════════════════════════════════════════════════
 * 📦 MAIN-FUNKTION
 * Der Einstiegspunkt der Anwendung.
 *
 * 1. .env laden (dotenv)
 * 2. DATABASE_URL lesen (mit Fallback)
 * 3. PostgreSQL-Pool erstellen
 * 4. Monitoring-Loop in Background spawnen
 * 5. Router mit CORS konfigurieren
 * 6. Server auf Port 3000 starten
 *
 * 🎓 LERN-TIPP: Achte auf die Reihenfolge:
 *    State → CORS → Router → Layer → Serve
 * ═══════════════════════════════════════════════════════ */
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin@localhost:5432/mydb".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database");

    let monitor_pool = pool.clone();
    tokio::spawn(async move {
        async_services::run_monitoring_loop(monitor_pool).await;
    });
    println!("Monitoring loop started");

    let state = Arc::new(AppState { pool });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

use sqlx::postgres::PgPoolOptions;

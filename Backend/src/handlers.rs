// ─── Axum-Handler: Jede Funktion bearbeitet eine HTTP-Route ───
// Handler extrahieren den Shared State (AppState via Arc) und den Request-Body,
// delegieren die Business-Logik an async_services und geben JSON oder Fehler zurück.

use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::service_modules::async_services;
use crate::types::{
    AddEndpointReq, AppState, ChangeEmailReq, ChangePasswordReq,
    CreateAccountReq, DeleteAccountReq, DeleteEndpointReq, ErrorRes,
    IdParam, LoginReq, LoginRes, SetIntervallReq, UpdateEndpointReq,
};

// GET /acm – einfacher Healthcheck
pub async fn handle_healthcheck() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "message": "ACM API Connection Monitor" }))
}

// POST /acm/createAccount – neuen Benutzer registrieren
// 1. Prüft, ob die E-Mail bereits existiert (409 Conflict)
// 2. Hash das Passwort mit bcrypt
// 3. Speichert User in der DB und gibt userid + email zurück
pub async fn handle_create_account(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateAccountReq>,
) -> Result<Json<LoginRes>, (StatusCode, Json<ErrorRes>)> {
    // Prüfen, ob E-Mail bereits vergeben
    let existing = async_services::get_user_by_email(&state.pool, &body.email)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorRes {
                error: "Email already exists".to_string(),
            }),
        ));
    }

    // Passwort hashen – bcrypt mit Default-Cost (ca. 10-12 Runden)
    let hashed = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    // User in DB anlegen
    let user = async_services::create_account(&state.pool, &body.email, &hashed)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    Ok(Json(LoginRes {
        userid: user.userid,
        emailadress: user.emailadress,
    }))
}

// POST /acm/login – Benutzer anmelden
// 1. User per E-Mail suchen (401 wenn nicht gefunden)
// 2. Passwort mit bcrypt verifizieren
// 3. Bei Erfolg userid + email zurückgeben
pub async fn handle_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginReq>,
) -> Result<Json<LoginRes>, (StatusCode, Json<ErrorRes>)> {
    // User per E-Mail suchen – .ok_or() wandelt None in 401 Unauthorized um
    let user = async_services::get_user_by_email(&state.pool, &body.email)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ErrorRes {
                error: "Invalid email or password".to_string(),
            }),
        ))?;

    // Passwort-Validierung (Plaintext mit bcrypt-Hash vergleichen)
    let valid = bcrypt::verify(&body.password, &user.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
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

// GET /acm/home?userid=<id> – alle Endpunkte eines Users mit Status-Infos
// Gibt EndpointExtended-Liste zurück (Status, Interval, Sparkline-Daten)
pub async fn handle_home(
    State(state): State<Arc<AppState>>,
    Query(params): Query<IdParam>,
) -> Result<Json<Vec<async_services::EndpointExtended>>, (StatusCode, Json<ErrorRes>)> {
    let endpoints = async_services::get_user_endpoints(&state.pool, params.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(endpoints))
}

// GET /acm/user?userid=<id> – Benutzerdaten abrufen
pub async fn handle_user(
    State(state): State<Arc<AppState>>,
    Query(params): Query<IdParam>,
) -> Result<Json<async_services::User>, (StatusCode, Json<ErrorRes>)> {
    let user = async_services::get_user_by_id(&state.pool, params.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(user))
}

// PUT /acm/user/changePassword – Passwort ändern
// 1. Altes Passwort verifizieren
// 2. Neues Passwort hashen und speichern
pub async fn handle_change_password(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChangePasswordReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    // User laden, um altes Passwort zu prüfen
    let user = async_services::get_user_by_id(&state.pool, body.userid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    // Altes Passwort verifizieren
    let valid = bcrypt::verify(&body.old_password, &user.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorRes {
                error: "Current password is incorrect".to_string(),
            }),
        ));
    }

    // Neues Passwort hashen und speichern
    let hashed = bcrypt::hash(&body.new_password, bcrypt::DEFAULT_COST).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;

    async_services::change_password(&state.pool, body.userid, &hashed)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// PUT /acm/user/changeEmail – E-Mail-Adresse ändern
pub async fn handle_change_email(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChangeEmailReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    async_services::change_email(&state.pool, body.userid, &body.new_email)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// DELETE /acm/user/deleteAccount – Benutzerkonto löschen
pub async fn handle_delete_account(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DeleteAccountReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    async_services::delete_account(&state.pool, body.userid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// PUT /acm/addEndpoint – neuen Endpoint hinzufügen
// Gibt die generierte endpointid zurück
pub async fn handle_add_endpoint(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddEndpointReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let endpointid = async_services::add_endpoint(&state.pool, body.userid, &body.url)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "endpointid": endpointid })))
}

// PUT /acm/setIntervall – Check-Intervall für einen Endpoint setzen/aktualisieren
pub async fn handle_set_intervall(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetIntervallReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    async_services::set_intervall(&state.pool, body.endpointid, body.seconds)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// PUT /acm/deleteConfirm – Endpoint und alle zugehörigen Daten löschen
pub async fn handle_delete_endpoint(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DeleteEndpointReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    async_services::delete_endpoint(&state.pool, body.endpointid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// GET /acm/log?endpointid=<id> – Log-Einträge für einen Endpoint abrufen
pub async fn handle_log(
    State(state): State<Arc<AppState>>,
    Query(params): Query<IdParam>,
) -> Result<Json<Vec<async_services::Log>>, (StatusCode, Json<ErrorRes>)> {
    let logs = async_services::get_log(&state.pool, params.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(logs))
}

// PUT /acm/updateEndpoint – URL eines Endpoints aktualisieren
// Loggt die Änderung (ohne Status, da es kein Monitor-Check ist)
pub async fn handle_update_endpoint(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateEndpointReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    // URL in der endpoint-Tabelle aktualisieren
    async_services::update_endpoint(&state.pool, body.endpointid, &body.url)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    // Log-Eintrag mit neuer URL (status = None, da kein Health-Check)
    async_services::insert_log(
        &state.pool,
        body.endpointid,
        None,
        Some(&body.url),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes { error: e.to_string() }),
        )
    })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

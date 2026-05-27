// ─── Axum-Handler: Jede Funktion bearbeitet eine HTTP-Route ───
// Öffentliche Routen: healthcheck, login, createAccount
// Geschützte Routen: alle anderen – erfordern Session-Token im Authorization-Header

use axum::{
    extract::{State, Query},
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;

use crate::service_modules::async_services;
use crate::types::{
    AddEndpointReq, AppState, ChangeEmailReq, ChangePasswordReq,
    CreateAccountReq, DeleteEndpointReq, ErrorRes,
    IdParam, LoginReq, LoginRes, SetIntervallReq, UpdateEndpointReq,
};

// ─── Hilfsfunktion: userid aus Session-Token extrahieren ───
// Liest den Authorization: Bearer <token>-Header und schlägt die userid nach.
// Gibt 401 bei fehlendem/ungültigem/abgelaufenem Token.

fn get_userid_from_token(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<i32, (StatusCode, Json<ErrorRes>)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorRes {
                    error: "Missing or invalid authorization header".to_string(),
                }),
            )
        })?;

    state
        .sessions
        .read()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes {
                    error: "Internal server error".to_string(),
                }),
            )
        })?
        .get(token)
        .copied()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorRes {
                    error: "Invalid or expired session token".to_string(),
                }),
            )
        })
}

// ════════════════════════════════════════════════════════════════
//  Öffentliche Routen (kein Token erforderlich)
// ════════════════════════════════════════════════════════════════

// GET /acm – einfacher Healthcheck
pub async fn handle_healthcheck() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "message": "ACM API Connection Monitor" }))
}

// POST /acm/createAccount – neuen Benutzer registrieren
// 1. Prüft, ob die E-Mail bereits existiert (409 Conflict)
// 2. Hasht das Passwort mit bcrypt
// 3. Speichert User in der DB
// 4. Erzeugt Session-Token und gibt userid + email + token zurück
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

    // Session-Token erzeugen und speichern
    let token = uuid::Uuid::new_v4().to_string();
    state.sessions.write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes {
                error: "Internal server error".to_string(),
            }),
        )
    })?.insert(token.clone(), user.userid);

    Ok(Json(LoginRes {
        userid: user.userid,
        emailadress: user.emailadress,
        token,
    }))
}

// POST /acm/login – Benutzer anmelden
// 1. User per E-Mail suchen (401 wenn nicht gefunden)
// 2. Passwort mit bcrypt verifizieren
// 3. Erzeugt Session-Token und gibt userid + email + token zurück
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

    // Session-Token erzeugen und speichern
    let token = uuid::Uuid::new_v4().to_string();
    state.sessions.write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorRes {
                error: "Internal server error".to_string(),
            }),
        )
    })?.insert(token.clone(), user.userid);

    Ok(Json(LoginRes {
        userid: user.userid,
        emailadress: user.emailadress,
        token,
    }))
}

// ════════════════════════════════════════════════════════════════
//  Geschützte Routen (Token erforderlich)
// ════════════════════════════════════════════════════════════════

// GET /acm/home – alle Endpunkte des eingeloggten Users mit Status-Infos
pub async fn handle_home(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<async_services::EndpointExtended>>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;
    let endpoints = async_services::get_user_endpoints(&state.pool, userid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(endpoints))
}

// GET /acm/user – Benutzerdaten des eingeloggten Users abrufen
pub async fn handle_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<async_services::User>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;
    let user = async_services::get_user_by_id(&state.pool, userid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(user))
}

// PUT /acm/user/changePassword – Passwort des eingeloggten Users ändern
pub async fn handle_change_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;

    // User laden, um altes Passwort zu prüfen
    let user = async_services::get_user_by_id(&state.pool, userid)
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

    async_services::change_password(&state.pool, userid, &hashed)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// PUT /acm/user/changeEmail – E-Mail-Adresse des eingeloggten Users ändern
pub async fn handle_change_email(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ChangeEmailReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;

    async_services::change_email(&state.pool, userid, &body.new_email)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// DELETE /acm/user/deleteAccount – Benutzerkonto des eingeloggten Users löschen
pub async fn handle_delete_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;

    async_services::delete_account(&state.pool, userid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// PUT /acm/addEndpoint – neuen Endpoint für den eingeloggten User hinzufügen
pub async fn handle_add_endpoint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<AddEndpointReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;

    let endpointid = async_services::add_endpoint(&state.pool, userid, &body.url, &body.check_type)
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
    headers: HeaderMap,
    Json(body): Json<SetIntervallReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;
    async_services::set_intervall(&state.pool, body.endpointid, userid, body.seconds)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                (StatusCode::FORBIDDEN, Json(ErrorRes { error: "Access denied".to_string() }))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorRes { error: e.to_string() }))
            }
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// PUT /acm/deleteConfirm – Endpoint und alle zugehörigen Daten löschen (nur eigener)
pub async fn handle_delete_endpoint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<DeleteEndpointReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;
    async_services::delete_endpoint(&state.pool, body.endpointid, userid)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                (StatusCode::FORBIDDEN, Json(ErrorRes { error: "Access denied".to_string() }))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorRes { error: e.to_string() }))
            }
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// GET /acm/log?id=<id> – Log-Einträge für einen Endpoint abrufen (nur eigener)
pub async fn handle_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<IdParam>,
) -> Result<Json<Vec<async_services::Log>>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;
    let logs = async_services::get_log(&state.pool, params.id, userid)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                (StatusCode::FORBIDDEN, Json(ErrorRes { error: "Access denied".to_string() }))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorRes { error: e.to_string() }))
            }
        })?;
    Ok(Json(logs))
}

// PUT /acm/updateEndpoint – URL eines Endpoints aktualisieren (nur eigener)
pub async fn handle_update_endpoint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<UpdateEndpointReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)> {
    let userid = get_userid_from_token(&headers, &state)?;
    // URL (und optional check_type) in der endpoint-Tabelle aktualisieren
    async_services::update_endpoint(&state.pool, body.endpointid, userid, &body.url, body.check_type.as_deref())
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                (StatusCode::FORBIDDEN, Json(ErrorRes { error: "Access denied".to_string() }))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorRes { error: e.to_string() }))
            }
        })?;
    // Log-Eintrag mit neuer URL (status = None, check_type = None da kein Health-Check)
    async_services::insert_log(&state.pool, body.endpointid, None, Some(&body.url), None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorRes { error: e.to_string() }),
            )
        })?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

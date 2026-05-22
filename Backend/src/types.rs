// ─── Shared State & Request/Response-Typen ───
// Alle Datenstrukturen für den HTTP-Datenaustausch.
// Deserialize: wird aus JSON-Request-Body geparst (von serde)
// Serialize:   wird in JSON-Response umgewandelt

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// Shared State: Wird per Arc<AppState> an alle Handler weitergereicht
pub struct AppState {
    pub pool: PgPool, // PostgreSQL-Verbindungspool
}

// ─── Request-Structs (werden aus dem JSON-Body deserialisiert) ───

#[derive(Deserialize)]
pub struct CreateAccountReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    pub userid: i32,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ChangeEmailReq {
    pub userid: i32,
    pub new_email: String,
}

#[derive(Deserialize)]
pub struct DeleteAccountReq {
    pub userid: i32,
}

#[derive(Deserialize)]
pub struct AddEndpointReq {
    pub userid: i32,
    pub url: String,
}

#[derive(Deserialize)]
pub struct SetIntervallReq {
    pub endpointid: i32,
    pub seconds: i32, // Check-Intervall in Sekunden
}

#[derive(Deserialize)]
pub struct DeleteEndpointReq {
    pub endpointid: i32,
}

#[derive(Deserialize)]
pub struct UpdateEndpointReq {
    pub endpointid: i32,
    pub url: String,
}

// Query-Parameter für GET-Requests (z. B. /acm/home?userid=5)
#[derive(Deserialize)]
pub struct IdParam {
    pub id: i32,
}

// ─── Response-Structs (werden in JSON serialisiert) ───

#[derive(Serialize)]
pub struct LoginRes {
    pub userid: i32,
    pub emailadress: String,
}

#[derive(Serialize)]
pub struct ErrorRes {
    pub error: String,
}

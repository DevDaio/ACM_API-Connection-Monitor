// ─── Shared State & Request/Response-Typen ───
// AppState: Wird per Arc an alle Handler weitergereicht
// sessions: In-Memory-Map für Session-Token → userid

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::RwLock;

// Shared State: Wird per Arc<AppState> an alle Handler weitergereicht
pub struct AppState {
    pub pool: PgPool,
    pub sessions: RwLock<HashMap<String, i32>>, // Session-Token → userid
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
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ChangeEmailReq {
    pub new_email: String,
}

// Kein Body mehr nötig – userid wird aus dem Session-Token extrahiert
// Siehe handle_delete_account

#[derive(Deserialize)]
pub struct AddEndpointReq {
    pub url: String,
    #[serde(default = "default_check_type")]
    pub check_type: String, // "http" oder "icmp"
}

fn default_check_type() -> String {
    "http".to_string()
}

#[derive(Deserialize)]
pub struct SetIntervallReq {
    pub endpointid: i32,
    pub seconds: i32,
}

#[derive(Deserialize)]
pub struct DeleteEndpointReq {
    pub endpointid: i32,
}

#[derive(Deserialize)]
pub struct UpdateEndpointReq {
    pub endpointid: i32,
    pub url: String,
    pub check_type: Option<String>, // None = nicht ändern
}

// Query-Parameter für GET-Requests (z. B. /acm/log?endpointid=5)
#[derive(Deserialize)]
pub struct IdParam {
    pub id: i32,
}

// ─── Response-Structs (werden in JSON serialisiert) ───

#[derive(Serialize)]
pub struct LoginRes {
    pub userid: i32,
    pub emailadress: String,
    pub token: String, // Session-Token für nachfolgende Requests
}

#[derive(Serialize)]
pub struct ErrorRes {
    pub error: String,
}

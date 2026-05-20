/* ═══════════════════════════════════════════════════════
 * 🎯 AUFGABE: Server + Router + Handler implementieren
 *
 * 📥 ERWARTETER INPUT:
 * - DATABASE_URL aus Umgebungsvariable
 * - JSON-Requests (Email, Passwort, Endpoint-Daten)
 *
 * 📤 ERWARTETER OUTPUT:
 * - HTTP-Server auf Port 3000
 * - 13 API-Endpoints unter /acm/
 *
 * 💭 HINWEISE:
 * - Nutze axum::Router für Routen
 * - Shared State via Arc<AppState>
 * - CORS für Cross-Origin-Zugriff
 * - tokio::spawn für Monitoring-Loop
 * - sqlx::PgPool für DB-Connection
 * - bcrypt für Passwort-Hashing
 *
 * ✅ TEST:
 * curl http://localhost:3000/acm
 * → {"status":"ok","message":"ACM API Connection Monitor"}
 * ═══════════════════════════════════════════════════════ */

mod service_modules;

// TODO: Importiere die benötigten Module
// - axum (Router, routing::get/post/put/delete, extract::Query, Json, extract::State)
// - serde (Deserialize, Serialize)
// - service_modules::async_services
// - sqlx::PgPool
// - std::sync::Arc
// - tower_http::cors (CorsLayer, Any)

// TODO: Definiere AppState mit pool: PgPool

// TODO: Definiere Request-Deserialize-Structs:
// - CreateAccountReq { email, password }
// - LoginReq { email, password }
// - ChangePasswordReq { userid, old_password, new_password }
// - ChangeEmailReq { userid, new_email }
// - DeleteAccountReq { userid }
// - AddEndpointReq { userid, url }
// - SetIntervallReq { endpointid, seconds }
// - DeleteEndpointReq { endpointid }
// - UpdateEndpointReq { endpointid, url }

// TODO: Definiere Query-Parameter-Struct IdParam { id: i32 }

// TODO: Definiere Response-Structs:
// - LoginRes { userid, emailadress }
// - ErrorRes { error }

// ─── HANDLER ───

// TODO: handle_healthcheck
// Gib {"status":"ok","message":"ACM API Connection Monitor"} zurück

// TODO: handle_create_account
// 1. Prüfe ob Email existiert (409 wenn ja)
// 2. Hashe Passwort mit bcrypt
// 3. Lege User an
// 4. Gib userid + email zurück

// TODO: handle_login
// 1. Finde User per Email (401 wenn nicht)
// 2. Verifiziere Passwort mit bcrypt (401 wenn falsch)
// 3. Gib userid + email zurück

// TODO: handle_home
// Hole alle Endpoints des Users (get_user_endpoints)

// TODO: handle_user
// Hole User-Daten (get_user_by_id)

// TODO: handle_change_password
// 1. Verifiziere altes Passwort
// 2. Hashe neues Passwort
// 3. Speichere in DB

// TODO: handle_change_email
// Ändere Email in DB

// TODO: handle_delete_account
// Lösche User aus DB

// TODO: handle_add_endpoint
// Lege Endpoint an + verknüpfe mit User

// TODO: handle_set_intervall
// Setze Intervall (Upsert)

// TODO: handle_delete_endpoint
// Lösche Endpoint + Logs + Intervall

// TODO: handle_update_endpoint
// Ändere URL

// TODO: handle_log
// Hole Logs eines Endpoints

// ─── MAIN ───

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // TODO: .env laden (dotenv::dotenv().ok())

    // TODO: DATABASE_URL lesen (env::var mit Fallback)

    // TODO: PostgreSQL Pool erstellen (PgPoolOptions)

    // TODO: Monitoring-Loop spawnen (tokio::spawn)

    // TODO: CORS konfigurieren (AllowOrigin::Any, Methoden, Headers)

    // TODO: Router bauen (alle .route()-Aufrufe)

    // TODO: Server binden + serve

    Ok(())
}

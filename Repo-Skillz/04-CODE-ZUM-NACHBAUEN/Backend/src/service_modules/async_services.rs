/* ═══════════════════════════════════════════════════════
 * 🎯 AUFGABE: DB-Queries + Monitoring-Loop implementieren
 *
 * 📥 ERWARTETER INPUT:
 * - PgPool (Connection-Pool)
 * - Query-Parameter
 *
 * 📤 ERWARTETER OUTPUT:
 * - User, EndpointExtended, Log, PgQueryResult
 *
 * 💭 HINWEISE:
 * - Nutze sqlx::FromRow für Struct-Mapping
 * - fetch_one, fetch_optional, fetch_all je nach Erwartung
 * - sqlx::query_as für Return-Typen
 * - sqlx::query für execute-only
 * - chrono::NaiveDate / NaiveTime für Datum/Zeit
 * - reqwest::Client für HTTP-Checks
 * - HashMap<i32, Instant> für Intervall-Tracking
 *
 * ✅ TEST:
 * Nach Implementierung sollte cargo build --release
 * ohne Fehler kompilieren.
 * ═══════════════════════════════════════════════════════ */

// TODO: Importe
// use sqlx::postgres::PgQueryResult;
// use sqlx::PgPool;
// use chrono::{NaiveDate, NaiveTime};
// use std::collections::HashMap;
// use std::time::{Duration, Instant};

// TODO: User-Struct (sqlx::FromRow, Serialize, Deserialize)
// Felder: userid: i32, emailadress: String, password: String

// TODO: Log-Struct (sqlx::FromRow, Serialize, Deserialize)
// Felder: endpointid: i32, status: bool, statusdate: NaiveDate, statustime: NaiveTime

// TODO: EndpointExtended-Struct (sqlx::FromRow, Serialize)
// Felder: endpointid, url, status (Option<bool>), statusdate, statustime,
//         duration_seconds (Option<i32>), interval_seconds (Option<i32>),
//         status_history (Option<Vec<bool>>)

// TODO: create_account
// INSERT INTO "user" RETURNING *

// TODO: get_user_endpoints (KOMPLEX)
// SELECT mit JOINs, LATERAL, Array-Aggregation, EPOCH-Berechnung

// TODO: get_user_by_id
// SELECT * FROM "user"

// TODO: get_user_by_email (Option<User>!)
// SELECT * FROM "user" WHERE emailadress = $1

// TODO: change_password
// UPDATE "user" SET password = $1 WHERE userid = $2

// TODO: change_email
// UPDATE "user" SET emailadress = $1 WHERE userid = $2

// TODO: delete_account
// DELETE FROM "user"

// TODO: add_endpoint
// 1. COUNT prüfen, ggf. Sequence reset
// 2. INSERT endpoint → RETURNING endpointid
// 3. INSERT userendpoint

// TODO: update_endpoint
// UPDATE endpoint SET url

// TODO: set_intervall (UPSERT!)
// INSERT ... ON CONFLICT DO UPDATE

// TODO: delete_endpoint
// Lösche in Reihenfolge: log → intervall → userendpoint → endpoint

// TODO: get_log
// SELECT ... ORDER BY statusdate DESC, statustime DESC

// TODO: insert_log (für Monitoring)
// INSERT INTO log (endpointid, status)

// TODO: EndpointInterval-Struct für Monitoring
// Felder: endpointid, seconds, url

// TODO: get_endpoints_with_intervals
// SELECT von intervall JOIN endpoint

// TODO: run_monitoring_loop
// loop { alle 5s: endpoints laden, prüfen, loggen }

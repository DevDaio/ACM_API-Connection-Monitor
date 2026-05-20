/* ═══════════════════════════════════════════════════════
 * 📦 DATENBANK-QUERIES + MONITORING-LOOP
 *
 * 🎯 ZWECK:
 * Enthält alle SQL-Queries und den asynchronen
 * Hintergrund-Loop, der Endpoints auf Erreichbarkeit
 * prüft.
 *
 * 📥 INPUT:
 * - PgPool (Connection-Pool)
 * - Query-Parameter (userid, email, endpointid, ...)
 *
 * 📤 OUTPUT:
 * - Rust-Structs (User, EndpointExtended, Log, ...)
 * - PgQueryResult für INSERT/UPDATE/DELETE
 *
 * 🔗 DEPENDENCIES:
 * - sqlx: PostgreSQL-Treiber
 * - chrono: Datum/Zeit-Typen
 * - reqwest: HTTP-Client für Monitoring
 * - tokio: Async-Sleep
 * - std::collections::HashMap: last_checked-Tracking
 *
 * 💡 KONZEPTE:
 * - sqlx::FromRow: Automatische DB→Struct-Mapping
 * - LATERAL JOIN: Letzter Log pro Endpoint
 * - Array-Aggregation: Status-History als Vec<bool>
 * - Upsert: ON CONFLICT DO UPDATE
 * - HashMap<i32, Instant>: Intervall-Tracking
 *
 * ⚠️ WICHTIG ZU WISSEN:
 * - get_user_by_email gibt Option<User> zurück
 *   (User kann nicht existieren)
 * - add_endpoint restartet die Sequence wenn Tabelle leer
 * - delete_endpoint löscht manuell in 4 Tabellen
 *   (weil CASCADE nur für FK, die nicht alle hier sind)
 *
 * 🎓 LERN-TIPP:
 * Die get_user_endpoints-Query ist die komplexeste im
 * Projekt. Studiere sie Zeile für Zeile.
 * ═══════════════════════════════════════════════════════ */

/* ═══════════════════════════════════════════════════════
 * 📦 IMPORTS
 * ═══════════════════════════════════════════════════════ */
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use chrono::{NaiveDate, NaiveTime};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/* ═══════════════════════════════════════════════════════
 * 📦 USER-STRUCT
 * Spiegelt die "user"-Tabelle wider.
 * sqlx::FromRow erlaubt automatisches Mapping aus SQL.
 * password wird als gehashter String gespeichert.
 * ═══════════════════════════════════════════════════════ */
#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub userid: i32,
    pub emailadress: String,
    pub password: String,
}

/* ═══════════════════════════════════════════════════════
 * 📦 LOG-STRUCT
 * Ein Eintrag im Monitoring-Log.
 * statusdate und statustime werden von der DB mit
 * DEFAULT CURRENT_DATE / CURRENT_TIME gesetzt.
 * ═══════════════════════════════════════════════════════ */
#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub endpointid: i32,
    pub status: bool,
    pub statusdate: NaiveDate,
    pub statustime: NaiveTime,
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: create_account
 * INSERT mit RETURNING * → gibt den kompletten User zurück
 * ═══════════════════════════════════════════════════════ */
pub async fn create_account(pool: &PgPool, email: &str, password: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO \"user\" (emailadress, password) VALUES ($1, $2) RETURNING *"
    )
    .bind(email)
    .bind(password)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/* ═══════════════════════════════════════════════════════
 * 📦 ENDPOINT-EXTENDED-STRUCT
 * Enthält alle Infos, die das Frontend braucht:
 * - Aktueller Status + Zeitstempel
 * - Uptime-Dauer in Sekunden
 * - Prüfintervall
 * - Letzte 30 Status-Werte als Sparkline-Daten
 *
 * 💡 LERN-TIPP: duration_seconds wird via SQL berechnet:
 *    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - letzter_change))
 * ═══════════════════════════════════════════════════════ */
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct EndpointExtended {
    pub endpointid: i32,
    pub url: String,
    pub status: Option<bool>,
    pub statusdate: Option<NaiveDate>,
    pub statustime: Option<NaiveTime>,
    pub duration_seconds: Option<i32>,
    pub interval_seconds: Option<i32>,
    pub status_history: Option<Vec<bool>>,
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: get_user_endpoints
 * ⭐ KOMPLEXESTE QUERY IM PROJEKT
 *
 * Holt alle Endpoints eines Users mit:
 * - JOIN über userendpoint (M:N)
 * - LEFT JOIN intervall (kann NULL sein)
 * - LEFT JOIN LATERAL (letzter Log-Eintrag)
 * - Berechneter duration_seconds (CURRENT_TIMESTAMP - letzter Status-Wechsel)
 * - Array-Aggregation der letzten 30 Status-Werte
 * ═══════════════════════════════════════════════════════ */
pub async fn get_user_endpoints(pool: &PgPool, userid: i32) -> Result<Vec<EndpointExtended>, sqlx::Error> {
    sqlx::query_as::<_, EndpointExtended>(
        "SELECT e.endpointid, e.url, l.status, l.statusdate, l.statustime, \
                CASE WHEN l.statusdate IS NOT NULL THEN \
                  EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - ( \
                    SELECT COALESCE( \
                      MAX(l2.statusdate + l2.statustime), \
                      (SELECT MIN(l3.statusdate + l3.statustime) FROM log l3 \
                       WHERE l3.endpointid = e.endpointid) \
                    ) FROM log l2 \
                    WHERE l2.endpointid = e.endpointid \
                      AND l2.status != l.status \
                  )))::integer \
                ELSE NULL END AS duration_seconds, \
                i.seconds AS interval_seconds, \
                ARRAY(SELECT status FROM log WHERE endpointid = e.endpointid \
                      ORDER BY statusdate ASC, statustime ASC LIMIT 30) AS status_history \
         FROM endpoint e \
         JOIN userendpoint ue ON e.endpointid = ue.endpointid \
         LEFT JOIN intervall i ON i.endpointid = e.endpointid \
         LEFT JOIN LATERAL ( \
             SELECT status, statusdate, statustime \
             FROM log \
             WHERE endpointid = e.endpointid \
             ORDER BY statusdate DESC, statustime DESC \
             LIMIT 1 \
         ) l ON true \
         WHERE ue.userid = $1"
    )
    .bind(userid)
    .fetch_all(pool)
    .await
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: get_user_by_id
 * Holt einen User per Primärschlüssel.
 * ═══════════════════════════════════════════════════════ */
pub async fn get_user_by_id(pool: &PgPool, userid: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM \"user\" WHERE userid = $1"
    )
    .bind(userid)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: get_user_by_email
 * Holt User per Email. fetch_optional → Option<User>
 * ═══════════════════════════════════════════════════════ */
pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM \"user\" WHERE emailadress = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: change_password
 * ═══════════════════════════════════════════════════════ */
pub async fn change_password(pool: &PgPool, userid: i32, new_password: &str) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "UPDATE \"user\" SET password = $1 WHERE userid = $2"
    )
    .bind(new_password)
    .bind(userid)
    .execute(pool)
    .await?;
    Ok(rows)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: change_email
 * ═══════════════════════════════════════════════════════ */
pub async fn change_email(pool: &PgPool, userid: i32, new_email: &str) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "UPDATE \"user\" SET emailadress = $1 WHERE userid = $2"
    )
    .bind(new_email)
    .bind(userid)
    .execute(pool)
    .await?;
    Ok(rows)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: delete_account
 * CASCADE DELETE in der DB löscht verknüpfte Einträge.
 * ═══════════════════════════════════════════════════════ */
pub async fn delete_account(pool: &PgPool, userid: i32) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "DELETE FROM \"user\" WHERE userid = $1"
    )
    .bind(userid)
    .execute(pool)
    .await?;
    Ok(rows)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: add_endpoint
 *
 * 1. Zählt vorhandene Endpoints
 * 2. Falls Tabelle leer: RESTART IDENTITY (Sequence zurücksetzen)
 * 3. INSERT Endpoint → RETURNING endpointid
 * 4. INSERT userendpoint (Verknüpfung)
 *
 * 💡 Warum Sequence-Reset? Damit die IDs immer bei 1
 *    starten, wenn keine Daten da sind (Sauberkeit).
 * ═══════════════════════════════════════════════════════ */
pub async fn add_endpoint(pool: &PgPool, userid: i32, url: &str) -> Result<i32, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM endpoint")
        .fetch_one(pool)
        .await?;
    if count.0 == 0 {
        sqlx::query("ALTER TABLE endpoint ALTER COLUMN endpointid RESTART WITH 1")
            .execute(pool)
            .await?;
    }
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO endpoint (url) VALUES ($1) RETURNING endpointid"
    )
    .bind(url)
    .fetch_one(pool)
    .await?;
    let endpointid = row.0;
    sqlx::query(
        "INSERT INTO userendpoint (userid, endpointid) VALUES ($1, $2)"
    )
    .bind(userid)
    .bind(endpointid)
    .execute(pool)
    .await?;
    Ok(endpointid)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: update_endpoint
 * Ändert die URL eines bestehenden Endpoints.
 * ═══════════════════════════════════════════════════════ */
pub async fn update_endpoint(pool: &PgPool, endpointid: i32, url: &str) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("UPDATE endpoint SET url = $1 WHERE endpointid = $2")
        .bind(url)
        .bind(endpointid)
        .execute(pool)
        .await
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: set_intervall
 * UPSERT: INSERT oder UPDATE (wenn bereits vorhanden)
 * ═══════════════════════════════════════════════════════ */
pub async fn set_intervall(pool: &PgPool, endpointid: i32, seconds: i32) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "INSERT INTO intervall (endpointid, seconds) VALUES ($1, $2) \
         ON CONFLICT (endpointid) DO UPDATE SET seconds = EXCLUDED.seconds"
    )
    .bind(endpointid)
    .bind(seconds)
    .execute(pool)
    .await?;
    Ok(rows)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: delete_endpoint
 * Manuelles Löschen in 4 Schritten (weil nicht alle
 * FKs CASCADE haben):
 * 1. log-Einträge löschen
 * 2. intervall löschen
 * 3. userendpoint-Verknüpfung löschen
 * 4. endpoint selbst löschen
 * ═══════════════════════════════════════════════════════ */
pub async fn delete_endpoint(pool: &PgPool, endpointid: i32) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM log WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM intervall WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM userendpoint WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    let rows = sqlx::query("DELETE FROM endpoint WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    Ok(rows)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: get_log
 * Holt alle Logs eines Endpoints (neueste zuerst).
 * ═══════════════════════════════════════════════════════ */
pub async fn get_log(pool: &PgPool, endpointid: i32) -> Result<Vec<Log>, sqlx::Error> {
    let logs = sqlx::query_as::<_, Log>(
        "SELECT * FROM log WHERE endpointid = $1 \
         ORDER BY statusdate DESC, statustime DESC"
    )
    .bind(endpointid)
    .fetch_all(pool)
    .await?;
    Ok(logs)
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: insert_log
 * Wird vom Monitoring-Loop aufgerufen.
 * statusdate/statustime werden automatisch von der DB
 * gesetzt (DEFAULT CURRENT_DATE / CURRENT_TIME).
 * ═══════════════════════════════════════════════════════ */
pub async fn insert_log(pool: &PgPool, endpointid: i32, status: bool) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "INSERT INTO log (endpointid, status) VALUES ($1, $2)"
    )
    .bind(endpointid)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(rows)
}

/* ═══════════════════════════════════════════════════════
 * 📦 INTERVAL-STRUCT (Hilfstyp für Monitoring)
 * Nur intern verwendet: EndpointID + Intervall + URL
 * ═══════════════════════════════════════════════════════ */
#[derive(sqlx::FromRow)]
pub struct EndpointInterval {
    pub endpointid: i32,
    pub seconds: i32,
    pub url: String,
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: get_endpoints_with_intervals
 * Holt alle Endpoints, die ein Intervall gesetzt haben.
 * Nur diese werden vom Monitoring-Loop geprüft.
 * ═══════════════════════════════════════════════════════ */
pub async fn get_endpoints_with_intervals(pool: &PgPool) -> Result<Vec<EndpointInterval>, sqlx::Error> {
    sqlx::query_as::<_, EndpointInterval>(
        "SELECT i.endpointid, i.seconds, e.url \
         FROM intervall i \
         JOIN endpoint e ON e.endpointid = i.endpointid"
    )
    .fetch_all(pool)
    .await
}

/* ═══════════════════════════════════════════════════════
 * 📦 FUNKTION: run_monitoring_loop
 * ⭐ HERZ DES MONITORINGS
 *
 * Läuft in einem eigenem Tokio-Task (Background).
 *
 * Ablauf:
 * 1. Alle 5 Sekunden aufwachen
 * 2. Alle Endpoints mit Intervall aus DB laden
 * 3. Für jeden: Prüfen ob Intervall abgelaufen
 * 4. Wenn fällig: HTTP GET → Status ermitteln
 * 5. Status in DB loggen
 *
 * last_checked: HashMap<i32, Instant> merkt sich,
 * wann jeder Endpoint zuletzt geprüft wurde.
 *
 * ⚠️ Fehlerbehandlung: Bei DB-Fehlern 10s warten
 * und neu versuchen (kein Crash).
 *
 * 🎓 LERN-TIPP: Das ist der einzige Teil des Backends,
 * der nicht von HTTP-Requests angetriggert wird.
 * ═══════════════════════════════════════════════════════ */
pub async fn run_monitoring_loop(pool: PgPool) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build reqwest client");

    let mut last_checked: HashMap<i32, Instant> = HashMap::new();

    loop {
        let endpoints = match get_endpoints_with_intervals(&pool).await {
            Ok(eps) => eps,
            Err(e) => {
                eprintln!("[Monitor] DB error: {e}");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        for ep in &endpoints {
            let should_check = match last_checked.get(&ep.endpointid) {
                Some(last) => last.elapsed() >= Duration::from_secs(ep.seconds as u64),
                None => true,
            };

            if !should_check {
                continue;
            }

            let status = match client.get(&ep.url).send().await {
                Ok(resp) => resp.status().is_success(),
                Err(_) => false,
            };

            if let Err(e) = insert_log(&pool, ep.endpointid, status).await {
                eprintln!("[Monitor] Log insert error for ep {}: {e}", ep.endpointid);
            }

            last_checked.insert(ep.endpointid, Instant::now());
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

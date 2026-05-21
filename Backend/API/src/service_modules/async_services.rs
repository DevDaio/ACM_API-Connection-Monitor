// ─── Externe Abhängigkeiten ───
// sqlx: PostgreSQL-Client, PgQueryResult für Rückgabewerte von INSERT/UPDATE/DELETE
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
// chrono: Datum/Zeit-Typen für die Log-Tabelle (statusdate DATE, statustime TIME)
use chrono::{NaiveDate, NaiveTime};
// HashMap für Last-Checked-Zeiten, Duration + Instant für Intervall-Prüfung
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ════════════════════════════════════════════════════════════════
//  Datenbank-Modelle (entsprechen den SQL-Tabellen)
// ════════════════════════════════════════════════════════════════

// #[derive(sqlx::FromRow)] ermöglicht, dass sqlx Zeilen aus der DB direkt in dieses Struct parst
// Die Feldnamen muessen exakt den Spaltennamen in der DB entsprechen
#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub userid: i32,
    pub emailadress: String,
    pub password: String, // gehashter Passwort-String (bcrypt)
}

// Log-Eintrag: speichert Status (up/down) mit Zeitstempel
#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub endpointid: i32,
    pub status: bool,           // true = up, false = down
    pub statusdate: NaiveDate,  // Datum des Status-Checks
    pub statustime: NaiveTime,  // Uhrzeit des Status-Checks
}

// ════════════════════════════════════════════════════════════════
//  Account-Funktionen
// ════════════════════════════════════════════════════════════════

// Erstellt einen neuen User in der "user"-Tabelle.
// RETURNING * gibt die komplette eingefügte Zeile zurück (inkl. der automatisch generierten userid)
// $1, $2: Platzhalter für bind-Werte (sqlx schützt vor SQL-Injection)
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

// ════════════════════════════════════════════════════════════════
//  Erweiterter Endpoint-Datentyp (mit JOIN-Feldern)
// ════════════════════════════════════════════════════════════════

// EndpointExtended: Ein Endpoint mit allen zusätzlichen Informationen,
// die das Dashboard braucht. Wird durch einen komplexen JOIN-Query befüllt.
// Option<T> bedeutet, dass der Wert NULL sein kann (z. B. wenn noch nie gepingt wurde)
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct EndpointExtended {
    pub endpointid: i32,
    pub url: String,
    pub status: Option<bool>,                   // letzter Status (NULL wenn noch nie gecheckt)
    pub statusdate: Option<NaiveDate>,          // Datum des letzten Status
    pub statustime: Option<NaiveTime>,          // Uhrzeit des letzten Status
    pub duration_seconds: Option<i32>,          // Sekunden seit dem letzten Statuswechsel
    pub interval_seconds: Option<i32>,          // eingestelltes Check-Intervall (NULL wenn keins)
    pub status_history: Option<Vec<bool>>,      // letzte 30 Status-Eintraege fuer Sparkline
}

// Holt alle Endpunkte eines Users mit aktuellen Status-Informationen.
// Der Query ist komplex – hier eine Erklärung der einzelnen Teile:
//   - e (endpoint) JOIN ue (userendpoint): verknüpft Endpunkte mit dem User
//   - LEFT JOIN i (intervall): Intervall ist optional (kann NULL sein)
//   - LEFT JOIN LATERAL l (... LIMIT 1): holt den neuesten Log-Eintrag pro Endpunkt
//   - duration_seconds: berechnet die Zeit seit dem letzten Statuswechsel per EXTRACT(EPOCH)
//   - status_history: sammelt die letzten 30 Statuswerte in ein Array (für Sparkline-Chart)
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

// ════════════════════════════════════════════════════════════════
//  User CRUD
// ════════════════════════════════════════════════════════════════

// Holt einen User anhand der userid (Primärschlüssel).
// Panickt (via fetch_one), wenn die ID nicht existiert.
pub async fn get_user_by_id(pool: &PgPool, userid: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM \"user\" WHERE userid = $1"
    )
    .bind(userid)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

// Holt einen User anhand der E-Mail.
// fetch_optional gibt Some(User) oder None zurück (statt Fehler bei Nichtexistenz).
pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM \"user\" WHERE emailadress = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

// Aktualisiert das Passwort eines Users
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

// Aktualisiert die E-Mail eines Users
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

// Löscht einen User (CASCADE löscht automatisch userendpoint, deren endpoints aber bleiben!)
// Achtung: Das Loeschen des Users entfernt nur die Verknuepfung (userendpoint),
// nicht die endpoint- oder log-Eintraege selbst.
pub async fn delete_account(pool: &PgPool, userid: i32) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "DELETE FROM \"user\" WHERE userid = $1"
    )
    .bind(userid)
    .execute(pool)
    .await?;
    Ok(rows)
}

// ════════════════════════════════════════════════════════════════
//  Endpoint CRUD
// ════════════════════════════════════════════════════════════════

// Fügt einen neuen Endpunkt hinzu und verknüpft ihn mit dem User.
// Wenn die endpoint-Tabelle leer ist, wird der Sequence-Counter zurückgesetzt (ALTER...RESTART).
// Das verhindert Lücken in der ID-Vergabe beim ersten Eintrag nach DB-Reset.
pub async fn add_endpoint(pool: &PgPool, userid: i32, url: &str) -> Result<i32, sqlx::Error> {
    // Prüfen, ob Tabelle leer ist – dann Sequence zurücksetzen
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM endpoint")
        .fetch_one(pool)
        .await?;
    if count.0 == 0 {
        sqlx::query("ALTER TABLE endpoint ALTER COLUMN endpointid RESTART WITH 1")
            .execute(pool)
            .await?;
    }
    // Endpoint in die Tabelle einfügen und die generierte ID zurückgeben
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO endpoint (url) VALUES ($1) RETURNING endpointid"
    )
    .bind(url)
    .fetch_one(pool)
    .await?;
    let endpointid = row.0;
    // Verknüpfung user <-> endpoint in der userendpoint-Tabelle
    sqlx::query(
        "INSERT INTO userendpoint (userid, endpointid) VALUES ($1, $2)"
    )
    .bind(userid)
    .bind(endpointid)
    .execute(pool)
    .await?;
    Ok(endpointid)
}

// Aktualisiert die URL eines Endpunkts
pub async fn update_endpoint(pool: &PgPool, endpointid: i32, url: &str) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("UPDATE endpoint SET url = $1 WHERE endpointid = $2")
        .bind(url)
        .bind(endpointid)
        .execute(pool)
        .await
}

// Setzt oder aktualisiert das Intervall für einen Endpunkt.
// ON CONFLICT (endpointid) DO UPDATE = UPSERT:
// Wenn bereits ein Eintrag fuer diese endpointid existiert, wird seconds ueberschrieben.
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

// Löscht einen Endpunkt und alle abhängigen Daten.
// Reihenfolge wichtig: log -> intervall -> userendpoint -> endpoint
// (Fremdschluessel-Constraints verhindern, dass ein endpoint geloescht wird,
//  solange noch abhaengige Zeilen in anderen Tabellen existieren)
pub async fn delete_endpoint(pool: &PgPool, endpointid: i32) -> Result<PgQueryResult, sqlx::Error> {
    // Zuerst Logs löschen (abhängige Zeilen)
    sqlx::query("DELETE FROM log WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    // Intervall löschen
    sqlx::query("DELETE FROM intervall WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    // User-Verknüpfung löschen
    sqlx::query("DELETE FROM userendpoint WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    // Endpoint selbst löschen
    let rows = sqlx::query("DELETE FROM endpoint WHERE endpointid = $1")
        .bind(endpointid)
        .execute(pool)
        .await?;
    Ok(rows)
}

// ════════════════════════════════════════════════════════════════
//  Log-Funktionen
// ════════════════════════════════════════════════════════════════

// Holt alle Log-Einträge für einen Endpunkt, absteigend sortiert (neueste zuerst)
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

// Fügt einen Log-Eintrag ein (status: true=up, false=down).
// statusdate und statustime werden automatisch per DEFAULT auf CURRENT_DATE/CURRENT_TIME gesetzt.
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

// ════════════════════════════════════════════════════════════════
//  Monitoring-Loop
// ════════════════════════════════════════════════════════════════

// Datenstruktur für einen Endpoint mit konfiguriertem Intervall
#[derive(sqlx::FromRow)]
pub struct EndpointInterval {
    pub endpointid: i32,
    pub seconds: i32,  // Check-Intervall in Sekunden
    pub url: String,
}

// Holt alle Endpunkte, die ein Intervall konfiguriert haben.
// Nur diese werden vom Monitor ueberwacht.
pub async fn get_endpoints_with_intervals(pool: &PgPool) -> Result<Vec<EndpointInterval>, sqlx::Error> {
    sqlx::query_as::<_, EndpointInterval>(
        "SELECT i.endpointid, i.seconds, e.url \
         FROM intervall i \
         JOIN endpoint e ON e.endpointid = i.endpointid"
    )
    .fetch_all(pool)
    .await
}

// Der zentrale Monitoring-Loop. Läuft in einem eigenen Tokio-Task.
// Alle 5 Sekunden werden alle Endpunkte mit Intervall überprüft.
// Ein Endpunkt wird nur dann angefragt, wenn sein Intervall abgelaufen ist.
pub async fn run_monitoring_loop(pool: PgPool) {
    // reqwest-Client: HTTP-Client mit 10s Timeout
    // danger_accept_invalid_certs(true): erlaubt Self-Signed-Zertifikate (nur für Entwicklung!)
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build reqwest client");

    // HashMap: endpointid -> letzter Check-Zeitpunkt (Instant)
    // Wird verwendet, um zu entscheiden, ob ein Endpunkt gepingt werden muss
    let mut last_checked: HashMap<i32, Instant> = HashMap::new();

    // Endlose Schleife – bricht nie ab (ausser bei schweren Fehlern)
    loop {
        // Hole alle Endpunkte mit Intervall aus der DB
        let endpoints = match get_endpoints_with_intervals(&pool).await {
            Ok(eps) => eps,
            Err(e) => {
                eprintln!("[Monitor] DB error: {e}");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        // Jeden Endpunkt einzeln checken
        for ep in &endpoints {
            // Prüfen, ob das Intervall abgelaufen ist
            // last_checked.get() gibt Option<&Instant> zurueck
            // None => wurde noch nie gecheckt => sofort checken
            // Some(last) => vergleiche elapsed() mit dem konfigurierten Intervall
            let should_check = match last_checked.get(&ep.endpointid) {
                Some(last) => last.elapsed() >= Duration::from_secs(ep.seconds as u64),
                None => true,
            };

            if !should_check {
                continue;
            }

            // HTTP-GET an die Ziel-URL senden
            // response.status().is_success() => 2xx-Statuscode
            // Err (z. B. Timeout, Connection-Refused) => false (Down)
            let status = match client.get(&ep.url).send().await {
                Ok(resp) => resp.status().is_success(),
                Err(_) => false,
            };

            // Status in der Log-Tabelle speichern
            if let Err(e) = insert_log(&pool, ep.endpointid, status).await {
                eprintln!("[Monitor] Log insert error for ep {}: {e}", ep.endpointid);
            }

            // letzten Check-Zeitpunkt aktualisieren
            last_checked.insert(ep.endpointid, Instant::now());
        }

        // 5 Sekunden warten, bevor der nächste Durchlauf startet
        // Das ist der "Takt" des Monitors – nicht zu verwechseln mit den Endpunkt-Intervallen
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

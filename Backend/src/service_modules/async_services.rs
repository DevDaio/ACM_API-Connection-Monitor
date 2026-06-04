// ─── Externe Abhängigkeiten ───
// sqlx: PostgreSQL-Client, PgQueryResult für Rückgabewerte von INSERT/UPDATE/DELETE
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
// chrono: Datum/Zeit-Typen für die Log-Tabelle (statusdate DATE, statustime TIME)
use chrono::{NaiveDate, NaiveTime};
// HashMap für Last-Checked-Zeiten, Duration + Instant für Intervall-Prüfung
use std::collections::HashMap;
use std::time::{Duration, Instant};
use futures::future::join_all;

// ════════════════════════════════════════════════════════════════
//  Datenbank-Modelle (entsprechen den SQL-Tabellen)
// ════════════════════════════════════════════════════════════════

// #[derive(sqlx::FromRow)] ermöglicht, dass sqlx Zeilen aus der DB direkt in dieses Struct parst
// Die Feldnamen müssen exakt den Spaltennamen in der DB entsprechen
#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub userid: i32,
    pub emailadress: String,
    pub password: String, // gehashter Passwort-String (bcrypt)
}

// Log-Eintrag: speichert Status (up/down) mit Zeitstempel
// url: die URL zum Zeitpunkt des Checks/Edits (NULL bei alten Einträgen)
// status: NULL wenn es ein Edit-Event ist (kein Monitor-Check)
#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub endpointid: i32,
    pub status: Option<bool>,    // true = up, false = down, NULL = URL-Edit
    pub statusdate: NaiveDate,  // Datum des Status-Checks
    pub statustime: NaiveTime,  // Uhrzeit des Status-Checks
    pub url: Option<String>,    // URL zum Zeitpunkt des Eintrags
    pub check_type: Option<String>, // "http", "tcp", "icmp", NULL = URL-Edit
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
    pub check_type: String,                      // "http", "tcp" oder "icmp"
    pub active: bool,                            // Killswitch: true = wird überwacht
    pub status: Option<bool>,                   // letzter Status (NULL wenn noch nie gecheckt)
    pub statusdate: Option<NaiveDate>,          // Datum des letzten Status
    pub statustime: Option<NaiveTime>,          // Uhrzeit des letzten Status
    pub duration_seconds: Option<i32>,          // Sekunden seit dem letzten Statuswechsel
    pub interval_seconds: Option<i32>,          // eingestelltes Check-Intervall (NULL wenn keins)
    pub status_history: Vec<bool>,               // letzte 30 Status-Eintraege fuer Sparkline
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
        "SELECT e.endpointid, e.url, e.check_type, e.active, l.status, l.statusdate, l.statustime, \
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
                COALESCE((SELECT ARRAY(SELECT status FROM (SELECT status, statusdate, statustime \
                      FROM log WHERE endpointid = e.endpointid AND status IS NOT NULL \
                      ORDER BY statusdate DESC, statustime DESC LIMIT 30 \
                ) sub ORDER BY statusdate ASC, statustime ASC)), ARRAY[]::BOOLEAN[]) AS status_history \
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
// Gibt Err(sqlx::Error::RowNotFound) zurück, wenn die ID nicht existiert.
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
// Achtung: Das Löschen des Users entfernt nur die Verknüpfung (userendpoint),
// nicht die endpoint- oder log-Einträge selbst.
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
// Die endpointid wird automatisch von GENERATED ALWAYS AS IDENTITY vergeben.
pub async fn add_endpoint(pool: &PgPool, userid: i32, url: &str, check_type: &str) -> Result<i32, sqlx::Error> {
    // Endpoint in die Tabelle einfügen und die generierte ID zurückgeben
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO endpoint (url, check_type) VALUES ($1, $2) RETURNING endpointid"
    )
    .bind(url)
    .bind(check_type)
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

// Aktualisiert die URL und optional check_type eines Endpunkts (nur wenn der User Eigentümer ist)
pub async fn update_endpoint(pool: &PgPool, endpointid: i32, userid: i32, url: &str, check_type: Option<&str>) -> Result<PgQueryResult, sqlx::Error> {
    // Prüfen, ob der User diesen Endpoint besitzt
    let owned = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM userendpoint WHERE endpointid = $1 AND userid = $2"
    )
    .bind(endpointid)
    .bind(userid)
    .fetch_optional(pool)
    .await?;
    if owned.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }

    match check_type {
        Some(ct) => {
            sqlx::query("UPDATE endpoint SET url = $1, check_type = $2 WHERE endpointid = $3")
                .bind(url)
                .bind(ct)
                .bind(endpointid)
                .execute(pool)
                .await
        }
        None => {
            sqlx::query("UPDATE endpoint SET url = $1 WHERE endpointid = $2")
                .bind(url)
                .bind(endpointid)
                .execute(pool)
                .await
        }
    }
}

// Setzt oder aktualisiert das Intervall für einen Endpunkt (nur wenn der User Eigentümer ist).
// ON CONFLICT (endpointid) DO UPDATE = UPSERT:
// Wenn bereits ein Eintrag für diese endpointid existiert, wird seconds überschrieben.
pub async fn set_intervall(pool: &PgPool, endpointid: i32, userid: i32, seconds: i32) -> Result<PgQueryResult, sqlx::Error> {
    // Prüfen, ob der User diesen Endpoint besitzt
    let owned = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM userendpoint WHERE endpointid = $1 AND userid = $2"
    )
    .bind(endpointid)
    .bind(userid)
    .fetch_optional(pool)
    .await?;
    if owned.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }

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

// Löscht einen Endpunkt und alle abhängigen Daten (nur wenn der User Eigentümer ist).
// Reihenfolge wichtig: log -> intervall -> userendpoint -> endpoint
// (ON DELETE CASCADE an den FK-Constraints würde automatisch löschen,
//  dennoch wird explizit in Reihenfolge gelöscht für Klarheit)
pub async fn delete_endpoint(pool: &PgPool, endpointid: i32, userid: i32) -> Result<PgQueryResult, sqlx::Error> {
    // Prüfen, ob der User diesen Endpoint besitzt
    let owned = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM userendpoint WHERE endpointid = $1 AND userid = $2"
    )
    .bind(endpointid)
    .bind(userid)
    .fetch_optional(pool)
    .await?;
    if owned.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }

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
    // User-Verknüpfung löschen (der eigenen, für den Fall dass andere User denselben Endpoint haben)
    sqlx::query("DELETE FROM userendpoint WHERE endpointid = $1 AND userid = $2")
        .bind(endpointid)
        .bind(userid)
        .execute(pool)
        .await?;
    // Endpoint selbst löschen (nur wenn keine userendpoint-Verknüpfungen mehr existieren)
    let rows = sqlx::query(
        "DELETE FROM endpoint WHERE endpointid = $1 AND NOT EXISTS \
         (SELECT 1 FROM userendpoint WHERE endpointid = $1)"
    )
    .bind(endpointid)
    .execute(pool)
    .await?;
    Ok(rows)
}

// Setzt den active-Status eines Endpunkts (Killswitch).
pub async fn toggle_endpoint(pool: &PgPool, endpointid: i32, userid: i32, active: bool) -> Result<PgQueryResult, sqlx::Error> {
    let owned = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM userendpoint WHERE endpointid = $1 AND userid = $2"
    )
    .bind(endpointid)
    .bind(userid)
    .fetch_optional(pool)
    .await?;
    if owned.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }

    let rows = sqlx::query(
        "UPDATE endpoint SET active = $1 WHERE endpointid = $2"
    )
    .bind(active)
    .bind(endpointid)
    .execute(pool)
    .await?;
    Ok(rows)
}

// ════════════════════════════════════════════════════════════════
//  Log-Funktionen
// ════════════════════════════════════════════════════════════════

// Holt alle Log-Einträge für einen Endpunkt (nur wenn der User Eigentümer ist), absteigend sortiert
pub async fn get_log(pool: &PgPool, endpointid: i32, userid: i32) -> Result<Vec<Log>, sqlx::Error> {
    // Prüfen, ob der User diesen Endpoint besitzt
    let owned = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM userendpoint WHERE endpointid = $1 AND userid = $2"
    )
    .bind(endpointid)
    .bind(userid)
    .fetch_optional(pool)
    .await?;
    if owned.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }

    let logs = sqlx::query_as::<_, Log>(
        "SELECT * FROM log WHERE endpointid = $1 \
         ORDER BY statusdate DESC, statustime DESC"
    )
    .bind(endpointid)
    .fetch_all(pool)
    .await?;
    Ok(logs)
}

// Fügt einen Log-Eintrag ein.
// status: Some(true/false) = Monitor-Check, None = URL-Edit
// url: wird mitgespeichert, damit Änderungen nachvollziehbar sind
pub async fn insert_log(pool: &PgPool, endpointid: i32, status: Option<bool>, url: Option<&str>, check_type: Option<&str>) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "INSERT INTO log (endpointid, status, url, check_type) VALUES ($1, $2, $3, $4)"
    )
    .bind(endpointid)
    .bind(status)
    .bind(url)
    .bind(check_type)
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
    pub check_type: String, // "http", "tcp" oder "icmp"
}

// Holt alle Endpunkte, die ein Intervall konfiguriert haben.
// Nur diese werden vom Monitor überwacht.
pub async fn get_endpoints_with_intervals(pool: &PgPool) -> Result<Vec<EndpointInterval>, sqlx::Error> {
    sqlx::query_as::<_, EndpointInterval>(
        "SELECT i.endpointid, i.seconds, e.url, e.check_type \
         FROM intervall i \
         JOIN endpoint e ON e.endpointid = i.endpointid \
         WHERE e.active = true"
    )
    .fetch_all(pool)
    .await
}

// TCP-Port-Check: prüft ob ein TCP-Port offen ist (connect mit 5s Timeout)
async fn tcp_ping(target: &str) -> bool {
    // Entferne http:// oder https:// Prefix falls vorhanden
    let addr = target
        .strip_prefix("http://")
        .or_else(|| target.strip_prefix("https://"))
        .and_then(|s| s.split('/').next())
        .unwrap_or(target);

    match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(addr),
    )
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

// ICMP-Ping via system ping (kein root nötig – system ping hat bereits CAP_NET_RAW)
async fn icmp_ping(target: &str) -> bool {
    let hostname = target
        .strip_prefix("http://")
        .or_else(|| target.strip_prefix("https://"))
        .and_then(|s| s.split('/').next())
        .and_then(|s| s.split(':').next())
        .unwrap_or(target);

    match tokio::process::Command::new("ping")
        .arg("-c1")
        .arg("-W3")
        .arg(hostname)
        .output()
        .await
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

// Der zentrale Monitoring-Loop. Läuft in einem eigenen Tokio-Task.
// Alle 1 Sekunde werden fällige Endpunkte parallel überprüft (join_all).
// Ein Endpunkt wird nur dann angefragt, wenn sein Intervall abgelaufen ist.
pub async fn run_monitoring_loop(pool: PgPool) {
    // reqwest-Client: HTTP-Client mit 5s Timeout
    // danger_accept_invalid_certs(true): erlaubt Self-Signed-Zertifikate (nur für Entwicklung!)
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
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

        // Fällige Endpoints filtern (sequentiell, read-only)
        let due: Vec<&EndpointInterval> = endpoints
            .iter()
            .filter(|ep| {
                match last_checked.get(&ep.endpointid) {
                    Some(last) => last.elapsed() >= Duration::from_secs(ep.seconds as u64),
                    None => true,
                }
            })
            .collect();

        // Alle fälligen Checks parallel ausführen
        let results = join_all(due.iter().map(|ep| {
            let client = client.clone();
            let url = ep.url.clone();
            let check_type = ep.check_type.clone();
            async move {
                let status = match check_type.as_str() {
                    "icmp" => icmp_ping(&url).await,
                    "tcp" => tcp_ping(&url).await,
                    _ => client
                        .get(&url)
                        .send()
                        .await
                        .map(|r| r.status().is_success())
                        .unwrap_or(false),
                };
                (ep.endpointid, status, url, check_type)
            }
        }))
        .await;

        // Ergebnisse sequentiell verarbeiten (Log + Timestamp)
        for (endpointid, status, url, check_type) in results {
            if let Err(e) = insert_log(&pool, endpointid, Some(status), Some(&url), Some(&check_type)).await {
                eprintln!("[Monitor] Log insert error for ep {}: {e}", endpointid);
            }
            last_checked.insert(endpointid, Instant::now());
        }

        // 1 Sekunde warten, bevor der nächste Durchlauf startet
        // Das ist der "Takt" des Monitors – nicht zu verwechseln mit den Endpunkt-Intervallen
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use chrono::{NaiveDate, NaiveTime};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub userid: i32,
    pub emailadress: String,
    pub password: String,
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub endpointid: i32,
    pub status: bool,
    pub statusdate: NaiveDate,
    pub statustime: NaiveTime,
}

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

pub async fn get_user_by_id(pool: &PgPool, userid: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM \"user\" WHERE userid = $1"
    )
    .bind(userid)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM \"user\" WHERE emailadress = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

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

pub async fn delete_account(pool: &PgPool, userid: i32) -> Result<PgQueryResult, sqlx::Error> {
    let rows = sqlx::query(
        "DELETE FROM \"user\" WHERE userid = $1"
    )
    .bind(userid)
    .execute(pool)
    .await?;
    Ok(rows)
}

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

pub async fn update_endpoint(pool: &PgPool, endpointid: i32, url: &str) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("UPDATE endpoint SET url = $1 WHERE endpointid = $2")
        .bind(url)
        .bind(endpointid)
        .execute(pool)
        .await
}

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

#[derive(sqlx::FromRow)]
pub struct EndpointInterval {
    pub endpointid: i32,
    pub seconds: i32,
    pub url: String,
}

pub async fn get_endpoints_with_intervals(pool: &PgPool) -> Result<Vec<EndpointInterval>, sqlx::Error> {
    sqlx::query_as::<_, EndpointInterval>(
        "SELECT i.endpointid, i.seconds, e.url \
         FROM intervall i \
         JOIN endpoint e ON e.endpointid = i.endpointid"
    )
    .fetch_all(pool)
    .await
}

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

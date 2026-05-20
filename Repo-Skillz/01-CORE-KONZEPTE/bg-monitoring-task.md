# Konzept: Background-Monitoring-Task

## Was ist das?
Ein asynchroner Hintergrund-Loop, der alle Endpoints in definierten Intervallen auf Erreichbarkeit prüft.

## Implementierung

```rust
let monitor_pool = pool.clone();
tokio::spawn(async move {
    async_services::run_monitoring_loop(monitor_pool).await;
});
```

### Der Loop (run_monitoring_loop)
```
loop (alle 5s aufwachen)
  └─ get_endpoints_with_intervals() → Vec<EndpointInterval>
  └─ für jeden Endpoint:
       ├─ last_checked prüfen (HashMap<i32, Instant>)
       ├─ if fällig:
       │    ├─ client.get(url).send().await
       │    ├─ status = response.status().is_success()
       │    └─ insert_log(endpointid, status)
       └─ last_checked aktualisieren
```

### Reqwest-Client
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))   // Timeout nach 10s
    .danger_accept_invalid_certs(true)  // Auch self-signed certs
    .build()?;
```

## Warum eigener Thread?
- Blockiert nicht den Haupt-Request-Handler
- Läuft unabhängig und parallel
- Einfach zu debuggen (eigene Fehlerbehandlung)

## Wichtige Details
- **Intervall-Prüfung**: `last_checked[ep.id] + seconds ≥ now` → check
- **Fehler-Toleranz**: DB-Fehler → 10s warten → retry
- **Kein Cron/Job-Queue**: Einfach, aber ausreichend

## Übungen
- 05-UEBUNGEN/Level-3-Fortgeschritten/01-monitoring-logik-optimieren.md

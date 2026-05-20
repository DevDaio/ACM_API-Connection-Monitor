# Übung: Eigenen API-Endpoint hinzufügen

**Level:** 2 – Mittel

## Aufgabe
Füge einen neuen API-Endpoint `GET /acm/stats` hinzu, der zurückgibt:
- `total_endpoints`: Anzahl aller Endpoints
- `total_users`: Anzahl aller User

## Schritte
1. **Neue Structs** in main.rs definieren (StatsRes)
2. **Handler** schreiben: `async fn handle_stats(state, pool)`
3. **Query** in async_services.rs: `async fn get_stats(pool) -> (i64, i64)`
4. **Route** in main() einfügen: `.route("/acm/stats", get(handle_stats))`

## SQL-Hilfe
```sql
SELECT (SELECT COUNT(*) FROM endpoint) AS total_endpoints,
       (SELECT COUNT(*) FROM "user") AS total_users
```

## Erwartetes Ergebnis
```json
{"total_endpoints": 5, "total_users": 3}
```

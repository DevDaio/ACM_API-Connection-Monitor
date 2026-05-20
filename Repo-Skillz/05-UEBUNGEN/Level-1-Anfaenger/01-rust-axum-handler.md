# Übung: Rust Axum Handler verstehen

**Level:** 1 – Anfänger

## Aufgabe
Lies den `handle_healthcheck`-Handler und erkläre, was er tut.

## Frage
Was gibt `GET /acm` zurück?

a) `{"status": "error"}`
b) `{"status": "ok", "message": "ACM API Connection Monitor"}`
c) `Hello World`
d) `404 Not Found`

## Lückentext
Ergänze die fehlenden Teile:

```rust
async fn handle_healthcheck() -> Json<__________> {
    Json(serde_json::json!({ "______": "ok", "message": "..." }))
}
```

## Lösung
b) und `serde_json::Value`, `"status"`

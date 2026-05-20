# Übung: CORS-Security-Fix

**Level:** 3 – Fortgeschritten

## Aufgabe
Aktuell erlaubt das Backend CORS von **Any** Origin. Das ist ein Sicherheitsrisiko.

## Aktueller Code
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any);
```

## Aufgaben
1. Erstelle eine `.env`-Variable `ALLOWED_ORIGIN` (Default: `http://localhost:8080`)
2. Lese sie in main.rs und setze sie als CORS-Origin
3. Für die Produktion soll nur die tatsächliche Domain erlaubt sein

## Beispiel-Lösung
```rust
let allowed_origin = std::env::var("ALLOWED_ORIGIN")
    .unwrap_or_else(|_| "http://localhost:8080".to_string());

let cors = CorsLayer::new()
    .allow_origin(allowed_origin.parse::<HeaderValue>().unwrap())
    // ...
```

## Bonus
Implementiere eine Whitelist mehrerer Origins (komma-getrennt in .env).

# Konzept: Rust + Axum REST-API

## Was ist das?
Axum ist ein modernes, async-freundliches Webframework für Rust. Es baut auf Tokio (async runtime) und Tower (Middleware-Stack) auf.

## Kernkonzepte in diesem Projekt

### Router
Ein Router verbindet Pfade mit Handlern:
```rust
let app = Router::new()
    .route("/acm", get(handle_healthcheck))
    .route("/acm/login", post(handle_login))
    .route("/acm/home", get(handle_home))
    .layer(cors)
    .with_state(state);
```

### State (AppState)
Geteilter Zustand, der an alle Handler übergeben wird:
```rust
struct AppState { pool: PgPool }
// per Arc<AppState> thread-sicher gemacht
let state = Arc::new(AppState { pool });
```

### Handler
Async-Funktionen, die aus dem State lesen:
```rust
async fn handle_home(
    State(state): State<Arc<AppState>>,
    Query(params): Query<IdParam>,
) -> Result<Json<Vec<...>>, (StatusCode, Json<ErrorRes>)> {
    // ...
}
```

### Error-Handling
Axum unterstützt `Result<Json<T>, (StatusCode, Json<ErrorRes>)>` für einheitliche Fehler.

### JSON-Serialisierung
Mit serde: `#[derive(Deserialize)]` für Requests, `#[derive(Serialize)]` für Responses.

## Warum Axum?
- Extrem schnell (zero-cost abstractions)
- Async von Grund auf (Tokio)
- Tower-Middleware (CORS, Auth, Logging)
- Typ-sicherer State und Extractors

## Verwendet in
- `Backend/API/src/main.rs` (Router + Handler)
- `Backend/API/src/service_modules/async_services.rs` (DB-Queries)

## Übungen
- 05-UEBUNGEN/Level-1-Anfaenger/01-rust-axum-handler.md
- 05-UEBUNGEN/Level-2-Mittel/01-eigenen-endpoint-hinzufuegen.md

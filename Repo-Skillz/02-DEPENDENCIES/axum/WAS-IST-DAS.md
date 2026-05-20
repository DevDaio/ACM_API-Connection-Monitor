# Axum 0.8.9

**Was macht es?** Async HTTP-Framework für Rust. Router, Handler, State-Management, Extractors.

**Warum?** Standard-Framework für async Rust-Web-APIs. Extrem performant, typ-sicher.

**Wo?** `Backend/API/src/main.rs` — Router-Definition, alle Handler

**Wie?**
- `Router::new().route("/path", get(handler))` — Routen definieren
- `State<Arc<AppState>>` — Shared State
- `Query<T>`, `Json<T>` — Parameter/JSON extrahieren
- `Result<Json<T>, (StatusCode, Json<Error>)>` — Response/Error

**Alternativen:** Actix-Web (älter, eigener Runtime), Rocket (magic annotations), Warp (Filter-basiert)

**Mini-Tutorial:**
```bash
cargo add axum --features macros
```
```rust
use axum::{Router, routing::get};

async fn hello() -> &'static str { "Hello" }

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

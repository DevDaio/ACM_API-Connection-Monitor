# tower-http 0.6

**Was macht es?** HTTP-Middleware für Tower/Axum: CORS, Kompression, Auth, Logging.

**Warum?** CORS ist nötig, weil Frontend und Backend verschiedene Ports haben (:8080 vs :3000).

**Wo?** `Backend/API/src/main.rs` — Zeilen 386-389

**Wie?**
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)          // Jeder Origin (Dev)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any);

app.layer(cors);
```

**Wichtig:** `AllowOrigin::Any` nur in Dev! In Produktion auf bekannte Origins einschränken.

**Alternativen:** Eigenes CORS-Handling (Fehleranfällig)

**Mini-Tutorial:**
```rust
use tower_http::cors::{CorsLayer, Any};
let cors = CorsLayer::new().allow_origin(Any);
```

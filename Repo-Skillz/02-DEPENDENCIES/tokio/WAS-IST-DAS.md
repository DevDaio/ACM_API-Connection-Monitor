# Tokio 1.52.3

**Was macht es?** Async-Runtime für Rust. Ermöglicht gleichzeitige, nicht-blockierende Ausführung.

**Warum?** Axum, sqlx, reqwest — alle brauchen Tokio. `tokio::spawn` für Background-Tasks.

**Wo?** `Backend/API/src/main.rs` (Runtime + Spawn), `async_services.rs` (run_monitoring_loop)

**Verwendete Konzepte:**
```rust
#[tokio::main]  // Main-Funktion in async konvertieren
tokio::spawn(async { ... });  // Hintergrund-Task starten
tokio::time::sleep(Duration::from_secs(5)).await;  // Nicht-blockierend warten
tokio::net::TcpListener::bind("0.0.0.0:3000").await;
```

**Features:** `full` (alles inkludiert)

**Alternativen:** async-std (weniger verbreitet), smol (minimalistisch)

**Mini-Tutorial:**
```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        println!("Hintergrund-Task");
    });
    handle.await.unwrap();
}
```

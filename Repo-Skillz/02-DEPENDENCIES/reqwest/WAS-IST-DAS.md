# reqwest 0.12

**Was macht es?** Async HTTP-Client für Rust.

**Warum?** Monitoring-Loop pingt API-Endpoints via HTTP GET.

**Wo?** `Backend/API/src/service_modules/async_services.rs` — Zeilen 237-268

**Wie?**
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .danger_accept_invalid_certs(true)
    .build()?;

let resp = client.get(&url).send().await;
let status = resp.map(|r| r.status().is_success()).unwrap_or(false);
```

**Wichtige Methoden:**
- `.timeout()` — Maximalzeit für Request
- `.danger_accept_invalid_certs(true)` — Auch self-signed SSL

**Alternativen:** ureq (synchron), hyper (low-level)

**Mini-Tutorial:**
```rust
let resp = reqwest::get("https://httpbin.org/get").await?;
println!("Status: {}", resp.status());
```

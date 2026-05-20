# chrono 0.4

**Was macht es?** Datums- und Zeit-Bibliothek für Rust.

**Warum?** SQL-Logs speichern NaiveDate und NaiveTime, sqlx braucht chrono-Typen.

**Wo?** `Backend/API/src/service_modules/async_services.rs` — Log-Struct und Query-Ergebnisse

**Typen:**
```rust
NaiveDate  // Datum ohne Zeitzone (2026-05-20)
NaiveTime  // Uhrzeit ohne Zeitzone (14:30:00)
```

**Features:** `serde` (für JSON-Serialisierung)

**Verwendet in:** Log-Struct (statusdate, statustime), get_user_endpoints (CURRENT_TIMESTAMP)

**Alternativen:** time (neuer, aber weniger verbreitet), jiff (modern)

**Mini-Tutorial:**
```rust
use chrono::NaiveDate;
let date = NaiveDate::parse_from_str("2026-05-20", "%Y-%m-%d").unwrap();
println!("{}", date.format("%d.%m.%Y")); // 20.05.2026
```

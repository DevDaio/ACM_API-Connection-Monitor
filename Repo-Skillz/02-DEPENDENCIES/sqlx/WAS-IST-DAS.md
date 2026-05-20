# sqlx 0.8

**Was macht es?** Async SQL-Bibliothek für Rust. Direktes SQL, typ-sichere Queries, Connection-Pooling.

**Warum?** Kein ORM — volle SQL-Kontrolle. Async, Postgres-native, Compile-Time-Checks optional.

**Wo?** `Backend/API/src/service_modules/async_services.rs` — alle DB-Queries

**Wie?**
```rust
// Query → Struct (mit FromRow derive)
let user = sqlx::query_as::<_, User>("SELECT * FROM \"user\" WHERE id = $1")
    .bind(userid)
    .fetch_one(&pool).await?;

// Execute (kein Return-Wert)
sqlx::query("DELETE FROM log WHERE endpointid = $1")
    .bind(endpointid)
    .execute(&pool).await?;

// Upsert
sqlx::query("INSERT INTO ... VALUES ($1, $2) ON CONFLICT DO UPDATE SET ...")
```

**Features:** postgres, runtime-tokio, tls-native-tls, chrono

**Alternativen:** Diesel (ORM, synchronous), SeaORM (async ORM), tokio-postgres (low-level)

**Mini-Tutorial:**
```bash
cargo add sqlx --features "runtime-tokio postgres chrono"
```
```rust
use sqlx::PgPool;
let pool = PgPool::connect("postgres://user:pass@localhost/db").await?;
let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;
```

# Konzept: PostgreSQL + sqlx

## Was ist das?
sqlx ist eine async Rust-Bibliothek für SQL-Datenbanken. Anders als ORMs schreibt man reines SQL und bekommt die Ergebnisse als typ-sichere Rust-Structs.

## In diesem Projekt

### Connection Pool
```rust
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

### Typ-sichere Queries
```rust
#[derive(sqlx::FromRow)]
pub struct User {
    pub userid: i32,
    pub emailadress: String,
    pub password: String,
}

// Automatisch in User-Struct deserialisieren
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM \"user\" WHERE emailadress = $1"
)
.bind(email)
.fetch_optional(pool)
.await?;
```

### Wichtige Patterns
| Pattern | Code |
|---------|------|
| INSERT + RETURNING | `INSERT INTO endpoint (url) VALUES ($1) RETURNING endpointid` |
| JOIN + LATERAL | `LEFT JOIN LATERAL (SELECT ... FROM log WHERE ... ORDER BY ... LIMIT 1) l ON true` |
| Array-Aggregation | `ARRAY(SELECT status FROM log WHERE ... LIMIT 30) AS status_history` |
| Upsert | `INSERT INTO intervall ... ON CONFLICT DO UPDATE` |
| CASCADE DELETE | `FOREIGN KEY ... ON DELETE CASCADE` |

### $1-Platzhalter
sqlx verwendet `$1, $2, ...` statt `?` (PostgreSQL-konform).

## Warum sqlx?
- Volles SQL (kein ORM-Voodoo)
- Async (Tokio-kompatibel)
- Typsicher (Compile-Time-Checks optional)
- Kein zusätzlicher Build-Step

## Übungen
- 05-UEBUNGEN/Level-2-Mittel/03-sql-query-schreiben.md

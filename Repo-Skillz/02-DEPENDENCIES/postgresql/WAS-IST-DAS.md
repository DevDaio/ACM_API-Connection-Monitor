# PostgreSQL 17

**Was macht es?** Relationale Open-Source-Datenbank. SQL-konform, JSON-Support, erweiterbar.

**Warum?** Robuste, bewährte DB. CASCADE DELETE, LATERAL JOIN, Array-Aggregation.

**Wo?** `DB/createTables.sql` und `docker-compose.yml`

**Verwendete Features:**
```sql
-- Auto-Increment (Postgres 10+)
userid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY

-- CASCADE DELETE
FOREIGN KEY (userid) REFERENCES "user"(userid) ON DELETE CASCADE

-- LATERAL JOIN (letzter Log-Eintrag pro Endpoint)
LEFT JOIN LATERAL (
    SELECT status FROM log WHERE endpointid = e.endpointid
    ORDER BY statusdate DESC LIMIT 1
) l ON true

-- Array aus Subquery
ARRAY(SELECT status FROM log WHERE ... ORDER BY ... LIMIT 30)

-- Upsert
INSERT INTO intervall ... ON CONFLICT (endpointid) DO UPDATE SET ...
```

**Tabellen:** user, endpoint, userendpoint (M:N), intervall, log

**Alternativen:** MySQL (weniger Features), SQLite (kein Netzwerk), CockroachDB (verteilt)

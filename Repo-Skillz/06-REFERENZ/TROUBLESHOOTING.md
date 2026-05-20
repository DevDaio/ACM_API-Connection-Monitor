# Troubleshooting

## Backend startet nicht

### "connection refused" bei PostgreSQL
```bash
# Prüfe ob PostgreSQL läuft
docker compose ps
# Logs anzeigen
docker compose logs postgres
```

### "DATABASE_URL not set"
```bash
# .env prüfen
cat .env
# Manuell setzen
export DATABASE_URL=postgres://admin:admin@localhost:5432/mydb
```

### Cargo build failed
```bash
# Prüfe Rust Version
rustc --version  # muss 1.75+ sein
# Target-Verzeichnis löschen
rm -rf Backend/API/target
```

## Frontend startet nicht

### "npm install failed"
```bash
# node_modules löschen
rm -rf Frontend/ACM_Frontend/node_modules
# Neu installieren
npm install
```

### "Vite not found"
```bash
# Prüfe Node.js Version
node --version  # muss 22+ sein
# Prüfe package.json
cat Frontend/ACM_Frontend/package.json
```

### API-Proxy funktioniert nicht
```bash
# Prüfe ob Backend läuft
curl http://localhost:3000/acm
# Vite-Proxy prüfen (vite.config.js)
# /acm → localhost:3000
```

## Monitoring-Loop

### "Keine Endpoints werden geprüft"
```sql
-- Prüfe ob Intervall gesetzt
SELECT * FROM intervall;
-- Prüfe ob Endpoints existieren
SELECT * FROM endpoint;
```

### "Falscher Status"
```bash
# Prüfe ob Endpoint erreichbar ist
curl -I https://dein-endpoint.com
# Monitoring-Logs prüfen
docker compose logs backend
```

## Docker

### Port conflict
```bash
# Prüfe ob Ports belegt sind
ss -tlnp | grep -E '8080|3000|5432'
# Andere Container stoppen
docker compose down
```

### Build failed
```bash
# Docker neu bauen (ohne Cache)
docker compose build --no-cache
```

## Datenbank

### "relation does not exist"
```sql
-- Tabellen anzeigen
\dt
-- createTables.sql manuell ausführen
\i DB/createTables.sql
```

### CASCADE DELETE funktioniert nicht
Prüfe ob alle FOREIGN KEYs `ON DELETE CASCADE` haben.

### Sequence-Problem
```sql
ALTER SEQUENCE "user_userid_seq" RESTART WITH 1;
```

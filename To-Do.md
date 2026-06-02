---

kanban-plugin: board

---

## To-Do

- [x] Authentifizierung von Front zu Backend (Session-Token via Authorization-Header)
- [x] CORS: `allow_headers(Any)` → explizite Liste `[AUTHORIZATION, CONTENT_TYPE, ACCEPT]` (Browser ignoriert `*`)
- [ ] Fehlerhandling einbauen (HTTP-Status-Codes, DB-Connection-Graceful-Shutdown, Input-Validierung)
- [ ] TCP-Ping: Default-Port fehlt → `tcp_ping()` schlägt fehl bei URLs ohne Port (z.B. `https://example.com`)

---

## Deployment (Aktuell)

- [x] Frontend: S3 Static Website (HTTP) + CloudFront (HTTPS)
- [x] Backend: EC2 + Nginx Reverse-Proxy + systemd-Service (Port 3000)
- [x] DB: RDS PostgreSQL
- [x] Elastic IP (feste Adresse für CloudFront-Origin)
- [ ] CloudFront: OAC-Konfiguration prüfen
- [ ] Nach Migration: CORS-Konfiguration verschlanken (same-origin)

---

%% kanban:settings
```
{"kanban-plugin":"board","list-collapse":[false]}
```
%%

---

kanban-plugin: board

---

## To-Do

- [x] Authentifizierung von Front zu Backend (Session-Token via Authorization-Header)
- [x] CORS: `allow_headers(Any)` → explizite Liste `[AUTHORIZATION, CONTENT_TYPE, ACCEPT]` (Browser ignoriert `*`)
- [x] CORS-Konfiguration obsolet (same-origin via Nginx-Reverse-Proxy)
- [ ] Fehlerhandling einbauen (HTTP-Status-Codes, DB-Connection-Graceful-Shutdown, Input-Validierung)

---

## Deployment (Aktuell)

- [x] Frontend: S3 Static Website via EC2 Nginx Reverse-Proxy
- [x] Backend: EC2 + Nginx Reverse-Proxy + systemd-Service (Port 3000)
- [x] DB: RDS PostgreSQL
- [x] Nginx: same-origin (kein CORS mehr nötig)

---

%% kanban:settings
```
{"kanban-plugin":"board","list-collapse":[false]}
```
%%

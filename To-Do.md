---

kanban-plugin: board

---

## To-Do

- [x] Authentifizierung von Front zu Backend (Session-Token via Authorization-Header)
- [ ] Fehlerhandling einbauen (HTTP-Status-Codes, DB-Connection-Graceful-Shutdown, Input-Validierung)
- [ ] TCP-Ping: Default-Port fehlt → `tcp_ping()` schlägt fehl bei URLs ohne Port (z.B. `https://example.com`)

---

%% kanban:settings
```
{"kanban-plugin":"board","list-collapse":[false]}
```
%%

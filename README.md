# ACM API Connection Monitor

Überwacht die Verfügbarkeit von API-Endpunkten in Echtzeit. Definiere Prüfintervalle, behalte Downtimes mit detaillierten Logs im Blick und erhalte sofortigen Status-Überblick über all deine Services.

## Features

- **User-Accounts** — Registrierung + Login mit Passwort-Hashing (bcrypt)
- **Session-Auth** — UUID-Token nach Login, geschützte Routen via `Authorization: Bearer`
- **Endpoint-Verwaltung** — Endpunkte hinzufügen, bearbeiten, löschen
- **Prüfintervalle** — Individuell pro Endpunkt (Sekunden/Minuten/Stunden)
- **Automatisches Monitoring** — Hintergrund-Task pingt alle Endpunkte im definierten Intervall
- **Live-Status** — Tabellarische Übersicht mit Status-LED, Uptime-Dauer, Sparkline-Chart
- **Detaillierte Logs** — Gefiltert nach Status (Up/Down) und Datum
- **Themes** — Farbschema umschaltbar (Lava Red / Hacker Green / Void Purple)
- **Spaceship-Cockpit-Design** — Brutalistisch-futuristisches HUD

## Tech-Stack

| Komponente | Technologie |
|---|---|
| **Frontend** | React 19 + Vite + Tailwind CSS v4 |
| **Backend** | Rust + Axum + Tokio + sqlx |
| **Datenbank** | PostgreSQL 17 |
| **Monitoring** | HTTP / TCP / ICMP im Hintergrund-Task |

## Schnellstart (Entwicklung)

**Voraussetzungen:** Rust, Node.js 22+, PostgreSQL 17 (lokal)

**1. Umgebungsvariablen konfigurieren**

Zwei `.env`-Dateien:

- **`/.env`** (Projekt-Root) — fürs Backend (wird von systemd geladen):
  ```bash
  DATABASE_URL=postgres://postgres:admin123!@localhost:5432/database-acm
  BACKEND_HOST=0.0.0.0
  BACKEND_PORT=3000
  RUST_LOG=info
  ```

- **`Frontend/.env`** — für Vite-Build (wichtig für Production!):
  ```bash
  VITE_API_URL=/acm
  FRONTEND_PORT=8080
  API_PROXY_TARGET=http://localhost:3000
  ```

**2. Backend starten**
```bash
cd Backend
cargo run
```

**3. Frontend starten (zweites Terminal)**
```bash
cd Frontend
npm ci
npm run dev
```

**4. Im Browser**
```bash
open http://localhost:8080
```

## API-Routes

| Methode | Route | Auth | Beschreibung |
|---|---|---|---|
| `GET` | `/acm` | ❌ | Healthcheck |
| `POST` | `/acm/login` | ❌ | Login (Email + Passwort) → Token |
| `POST` | `/acm/createAccount` | ❌ | Registrierung → Token |
| `GET` | `/acm/home` | ✅ Token | Eigene Endpoints (mit Status) |
| `GET` | `/acm/user` | ✅ Token | Eigene User-Daten |
| `PUT` | `/acm/user/changePassword` | ✅ Token | Passwort ändern |
| `PUT` | `/acm/user/changeEmail` | ✅ Token | Email ändern |
| `DELETE` | `/acm/user/deleteAccount` | ✅ Token | Account löschen |
| `PUT` | `/acm/addEndpoint` | ✅ Token | Neuen Endpoint hinzufügen (Body: `url`, `check_type`=http\|tcp\|icmp) |
| `PUT` | `/acm/updateEndpoint` | ✅ Token | Endpoint-URL ändern (Body: `endpointid`, `url`, `check_type` optional) |
| `PUT` | `/acm/setIntervall` | ✅ Token | Prüfintervall setzen |
| `PUT` | `/acm/deleteConfirm` | ✅ Token | Endpoint löschen |
| `GET` | `/acm/log?id=N` | ✅ Token | Log eines Endpoints |

> **Auth:** Geschützte Routen benötigen `Authorization: Bearer <token>` im Header.
> Der Token wird bei Login/Registrierung ausgestellt und gilt bis zum Backend-Neustart.

## Datenbank-Schema

```
user (userid, emailadress, password)
endpoint (endpointid, url, check_type)      # check_type = "http" | "tcp" | "icmp"
userendpoint (userid, endpointid)           # M:N-Verknüpfung
intervall (endpointid, seconds)             # Prüfintervall pro Endpoint
log (endpointid, status, statusdate, statustime, url, check_type)  # status/url/check_type nullable
```

## Projektstruktur

```
.
├── Backend/
│   ├── src/
│   │   ├── main.rs                 # Server-Setup, Router, CORS, DB-Init
│   │   ├── types.rs                # Request/Response-Structs + AppState
│   │   ├── handlers.rs             # Alle Route-Handler (async fns)
│   │   └── service_modules/
│   │       ├── mod.rs
│   │       └── async_services.rs   # Datenbank-Queries + Monitoring-Loop
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── Dockerfile                   # Multi-Stage Rust-Build
│
├── Frontend/
│   ├── src/
│   │   ├── App.jsx                 # Render-Template (State via useAppState-Hook)
│   │   ├── App.css                 # Tailwind-Import + Global-Styles
│   │   ├── main.jsx                # Einstiegspunkt (ReactDOM.createRoot)
│   │   ├── api.js                  # API-Client
│   │   ├── ThemeContext.jsx         # Theme-Provider
│   │   ├── index.css               # Theme-Vars + Utility-Classes
│   │   ├── assets/                 # Statische Assets
│   │   ├── hooks/
│   │   │   └── useAppState.js      # Gesamtes State-Management + Handler
│   │   ├── utils/
│   │   │   └── helpers.js          # Format-Funktionen + URL-Normalisierung
│   │   └── components/
│   │       ├── LandingPage.jsx     # Hero + Login
│   │       ├── Dashboard.jsx       # Hauptansicht
│   │       ├── EndpointCard.jsx    # Zeile in der Tabelle
│   │       ├── Modal.jsx           # Modal-Wrapper
│   │       ├── CreateAccountModal.jsx
│   │       ├── AddEndpointModal.jsx
│   │       ├── SetIntervallModal.jsx
│   │       ├── DeleteConfirmModal.jsx
│   │       ├── AccountSettingsModal.jsx
│   │       ├── LogModal.jsx        # Log mit Filter + Datum
│   │       ├── EditUrlModal.jsx    # URL bearbeiten
│   │       ├── Sparkline.jsx       # Mini-Chart
│   │       └── ThemeSwitcher.jsx   # Farbschema-Auswahl
│   ├── public/icons.svg
│   ├── index.html
│   ├── package.json
│   ├── vite.config.js
│   ├── eslint.config.js
│   ├── Dockerfile                  # Production-Build + nginx
│   └── nginx.conf                  # Reverse-Proxy für /acm → Backend
│
├── DB/
│   ├── createTables.sql            # DB-Init
│   └── data/                       # DB-Daten (lokal, gitignored)
│
├── setup.sh                        # Ein-Klick-Build
├── start-dev.sh                    # Backend + Frontend parallel starten
├── docker-compose.yml              # Postgres + Backend + Frontend
├── DEPLOY.md                       # AWS-Deployment-Anleitung
├── To-Do.md                        # Projekt-Tracking
├── explain-canvas.md               # Architektur-Diagramme
├── .env                            # Backend-Konfiguration (systemd)
└── Frontend/
    └── .env                        # Vite-Build-Konfiguration
```

## Reference

### API Routes → Handler → Frontend

| Methode | Route | Auth | Handler | Frontend-Aufruf |
|---|---|---|---|---|
| `GET` | `/acm` | ❌ | `handle_healthcheck()` | — |
| `POST` | `/acm/login` | ❌ | `handle_login()` | `api.login()` |
| `POST` | `/acm/createAccount` | ❌ | `handle_create_account()` | `api.createAccount()` |
| `GET` | `/acm/home` | ✅ | `handle_home()` | `api.getHome()` |
| `GET` | `/acm/user` | ✅ | `handle_user()` | `api.getUser()` |
| `PUT` | `/acm/user/changePassword` | ✅ | `handle_change_password()` | `api.changePassword()` |
| `PUT` | `/acm/user/changeEmail` | ✅ | `handle_change_email()` | `api.changeEmail()` |
| `DELETE` | `/acm/user/deleteAccount` | ✅ | `handle_delete_account()` | `api.deleteAccount()` |
| `PUT` | `/acm/addEndpoint` | ✅ | `handle_add_endpoint()` | `api.addEndpoint()` |
| `PUT` | `/acm/updateEndpoint` | ✅ | `handle_update_endpoint()` | `api.updateEndpoint()` |
| `PUT` | `/acm/setIntervall` | ✅ | `handle_set_intervall()` | `api.setIntervall()` |
| `PUT` | `/acm/deleteConfirm` | ✅ | `handle_delete_endpoint()` | `api.deleteEndpoint()` |
| `GET` | `/acm/log?id=N` | ✅ | `handle_log()` | `api.getLog()` |

### DB Tables → CREATE → Queries

| Tabelle | CREATE (main.rs) | Wichtige Queries (async_services.rs) |
|---|---|---|
| `"user"` | `main.rs:46` | `create_account()` · `get_user_by_email()` · `get_user_by_id()` · `change_password()` · `change_email()` · `delete_account()` |
| `endpoint` | `main.rs:47` (+ Migration `main.rs:56`) | `add_endpoint()` · `update_endpoint()` · `delete_endpoint()` · `get_user_endpoints()` (JOIN) |
| `userendpoint` | `main.rs:48` | `add_endpoint()` · `delete_endpoint()` |
| `intervall` | `main.rs:49` | `set_intervall()` · `get_endpoints_with_intervals()` |
| `log` | `main.rs:50` (+ Migration `main.rs:58`) | `insert_log()` · `get_log()` |

### Monitoring – Check-Methoden

| Methode | Funktion | Datei:Zeile | Protokoll |
|---|---|---|---|
| HTTP | `client.get().send().await` | `async_services.rs:422` | HTTP GET → 2xx? (5s Timeout) |
| TCP | `tcp_ping()` | `async_services.rs:334` | TCP-Verbindung (5s Timeout) |
| ICMP | `icmp_ping()` | `async_services.rs:354` | System `ping -c1 -W3` |

### Schlüssel-Types

| Type | File | Beschreibung |
|---|---|---|
| `AppState` | `types.rs:11` | Shared State (PgPool + Session-HashMap) |
| `EndpointExtended` | `async_services.rs:62` | Dashboard-Response (URL, Status, Sparkline, Intervall) |
| `EndpointInterval` | `async_services.rs:313` | Endpunkt + Intervall für Monitoring-Loop |
| `Log` | `async_services.rs:28` | Log-Eintrag (Status + Datum + URL + check_type) |

### Schlüssel-Komponenten (Frontend)

| Komponente | File | Aufgabe |
|---|---|---|
| `Dashboard` | `Dashboard.jsx` | Haupttabelle mit EndpointCards |
| `EndpointCard` | `EndpointCard.jsx` | Eine Zeile (Status-LED, URL, Badge, Buttons) |
| `AddEndpointModal` | `AddEndpointModal.jsx` | URL + check_type + Intervall eingeben |
| `EditUrlModal` | `EditUrlModal.jsx` | URL + check_type bearbeiten |
| `LogModal` | `LogModal.jsx` | Log-Tabelle mit Filter (Status + Methode + Datum) |
| `Modal` | `Modal.jsx` | Basis-Overlay (wide-Prop für Log) |
| `useAppState` | `hooks/useAppState.js` | Zentraler State + alle Callback-Handler |
| `api` | `api.js` | HTTP-Client mit Token-Management |
| `helpers` | `utils/helpers.js` | `normalizeUrl()`, `mapEndpoints()`, `fmtDuration()` |

## Deployment (AWS)

Siehe [DEPLOY.md](DEPLOY.md) für AWS:

- **Frontend:** S3 Static Website (HTTP) — gebaut mit Vite, deployed via `aws s3 sync`
- **Backend:** Rust/Axum auf EC2 — Port 3000, systemd-Service
- **Datenbank:** RDS PostgreSQL
- **Wichtig:** EC2-Public-IP wechselt bei Stop/Start → Frontend neu bauen + deployen nötig
- **CORS:** Explizite Header (Authorization, Content-Type, Accept) — kein `*`

## Entwickler

- **Creator:** T. Adickes
- **GitHub:** [DevDaio](https://github.com/DevDaio/)

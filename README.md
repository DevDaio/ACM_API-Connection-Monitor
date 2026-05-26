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
| **Monitoring** | reqwest (HTTP GET) im Hintergrund-Task |

## Schnellstart (Entwicklung)

**Voraussetzungen:** Rust, Node.js 22+, PostgreSQL 17 (lokal)

**1. Umgebungsvariablen konfigurieren**

`.env` im Projekt-Root anlegen (siehe `.env` als Vorlage) und Werte anpassen:
```bash
DATABASE_URL=postgres://postgres:admin123!@localhost:5432/database-acm
BACKEND_HOST=0.0.0.0
BACKEND_PORT=3000
FRONTEND_PORT=8080
API_PROXY_TARGET=http://localhost:3000
VITE_API_URL=/acm
```

**2. Backend starten**
```bash
cd Backend
cargo run
```

**3. Frontend starten (zweites Terminal)**
```bash
cd Frontend
npm install
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
| `PUT` | `/acm/addEndpoint` | ✅ Token | Neuen Endpoint hinzufügen |
| `PUT` | `/acm/updateEndpoint` | ✅ Token | Endpoint-URL ändern |
| `PUT` | `/acm/setIntervall` | ✅ Token | Prüfintervall setzen |
| `PUT` | `/acm/deleteConfirm` | ✅ Token | Endpoint löschen |
| `GET` | `/acm/log?id=N` | ✅ Token | Log eines Endpoints |

> **Auth:** Geschützte Routen benötigen `Authorization: Bearer <token>` im Header.
> Der Token wird bei Login/Registrierung ausgestellt und gilt bis zum Backend-Neustart.

## Datenbank-Schema

```
user (userid, emailadress, password)
endpoint (endpointid, url)
userendpoint (userid, endpointid)    # M:N-Verknüpfung
intervall (endpointid, seconds)      # Prüfintervall pro Endpoint
log (endpointid, status, statusdate, statustime)
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
│   └── Cargo.lock
│
├── Frontend/
│   ├── src/
│   │   ├── App.jsx                 # Render-Template (State via useAppState-Hook)
│   │   ├── api.js                  # API-Client
│   │   ├── ThemeContext.jsx         # Theme-Provider
│   │   ├── index.css               # Theme-Vars + Utility-Classes
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
│   └── eslint.config.js
│
├── DB/createTables.sql             # DB-Init
│
├── setup.sh                        # Ein-Klick-Build
├── DEPLOY.md                       # AWS-Deployment-Anleitung
└── .env                            # Konfiguration
```

## Deployment

Siehe [DEPLOY.md](DEPLOY.md) für AWS RDS + EC2 Step-by-Step und Umgebungsvariablen.

## Entwickler

- **Creator:** T. Adickes
- **GitHub:** [DevDaio](https://github.com/DevDaio/)

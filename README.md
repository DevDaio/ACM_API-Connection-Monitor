# ACM API Connection Monitor

Überwacht die Verfügbarkeit von API-Endpunkten in Echtzeit. Definiere Prüfintervalle, behalte Downtimes mit detaillierten Logs im Blick und erhalte sofortigen Status-Überblick über all deine Services.

## Features

- **User-Accounts** — Registrierung + Login mit Passwort-Hashing (bcrypt)
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
| **Container** | Docker + Docker Compose |

## Schnellstart (Entwicklung)

**Voraussetzungen:** Docker (für PostgreSQL), Rust, Node.js 22+

```bash
# 1. Datenbank starten
docker compose up -d postgres

# 2. Backend starten
cd Backend/API
DATABASE_URL=postgres://admin:admin@localhost:5432/mydb cargo run

# 3. Frontend starten (zweites Terminal)
cd Frontend/ACM_Frontend
npm install
npm run dev

# 4. Im Browser
open http://localhost:8080
```

## Deployment (Docker)

```bash
# 1. Backend bauen
cd Backend/API
cargo build --release
docker build -t acm-backend:latest -f Dockerfile.local .
cd ../..

# 2. Frontend bauen
cd Frontend/ACM_Frontend
npm ci && npm run build
docker build -t acm-frontend:latest .
cd ../..

# 3. Starten
docker compose up -d

# 4. Im Browser
open http://localhost
```

## API-Routes

| Methode | Route | Beschreibung |
|---|---|---|
| `GET` | `/acm` | Healthcheck |
| `POST` | `/acm/login` | Login (Email + Passwort) |
| `POST` | `/acm/createAccount` | Registrierung |
| `GET` | `/acm/home?id=N` | Endpoints eines Users (mit Status) |
| `GET` | `/acm/user?id=N` | User-Daten |
| `PUT` | `/acm/user/changePassword` | Passwort ändern |
| `PUT` | `/acm/user/changeEmail` | Email ändern |
| `DELETE` | `/acm/user/deleteAccount` | Account löschen |
| `PUT` | `/acm/addEndpoint` | Neuen Endpoint hinzufügen |
| `PUT` | `/acm/updateEndpoint` | Endpoint-URL ändern |
| `PUT` | `/acm/setIntervall` | Prüfintervall setzen |
| `PUT` | `/acm/deleteConfirm` | Endpoint löschen |
| `GET` | `/acm/log?id=N` | Log eines Endpoints |

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
├── Backend/API/
│   ├── src/
│   │   ├── main.rs                 # Server, Router, CORS, Handler
│   │   └── service_modules/
│   │       ├── mod.rs
│   │       └── async_services.rs   # Datenbank-Queries + Monitoring-Loop
│   ├── Dockerfile                  # Multi-Stage: Rust → Distroless
│   ├── Dockerfile.local            # Lokales Binary → Distroless
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── Frontend/ACM_Frontend/
│   ├── src/
│   │   ├── App.jsx                 # Haupt-App mit State-Management
│   │   ├── api.js                  # API-Client
│   │   ├── ThemeContext.jsx         # Theme-Provider
│   │   ├── index.css               # Theme-Vars + Utility-Classes
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
│   ├── Dockerfile                  # Nginx mit API-Proxy
│   └── package.json
│
├── DB/createTables.sql             # DB-Init
│
├── docker-compose.yml              # postgres + backend + frontend
├── setup.sh                        # Ein-Klick-Setup
├── DEPLOY.md                       # AWS + CI/CD-Anleitung
└── .env                            # Konfiguration
```

## Deployment

Siehe [DEPLOY.md](DEPLOY.md) für:
- AWS EC2 Step-by-Step
- GitHub Actions CI/CD
- Umgebungsvariablen

## Entwickler

- **Creator:** T. Adickes
- **GitHub:** [DevDaio](https://github.com/DevDaio/)

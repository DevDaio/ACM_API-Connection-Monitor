# Struktur-Map (Datei für Datei)

## Backend (Rust)

| Datei | Zeilen | Zweck |
|-------|--------|-------|
| `Backend/API/src/main.rs` | 415 | Server-Start, Router, alle Handler |
| `Backend/API/src/service_modules/mod.rs` | 2 | Modul-Deklaration |
| `Backend/API/src/service_modules/async_services.rs` | 279 | DB-Queries, Monitoring-Loop |
| `Backend/API/Cargo.toml` | 18 | Dependencies |
| `Backend/API/Dockerfile` | 21 | Multi-Stage Build |
| `Backend/API/Dockerfile.local` | - | Lokaler Build |

## Frontend (React)

| Datei | Zeilen | Zweck |
|-------|--------|-------|
| `Frontend/ACM_Frontend/src/main.jsx` | ~5 | Einstiegspunkt |
| `Frontend/ACM_Frontend/src/App.jsx` | 253 | State-Hub, Polling, Callbacks |
| `Frontend/ACM_Frontend/src/api.js` | 51 | API-Client (fetch) |
| `Frontend/ACM_Frontend/src/ThemeContext.jsx` | 38 | Theme-Provider (Context) |
| `Frontend/ACM_Frontend/src/index.css` | 61 | Globale Styles, Theme-Vars |
| `Frontend/ACM_Frontend/src/App.css` | - | Zusätzliche Styles |
| `Frontend/ACM_Frontend/src/components/LandingPage.jsx` | 115 | Login-Bildschirm |
| `Frontend/ACM_Frontend/src/components/Dashboard.jsx` | 132 | Hauptansicht mit Tabelle |
| `Frontend/ACM_Frontend/src/components/EndpointCard.jsx` | 75 | Tabellenzeile |
| `Frontend/ACM_Frontend/src/components/Modal.jsx` | 25 | Overlay-Wrapper |
| `Frontend/ACM_Frontend/src/components/CreateAccountModal.jsx` | 54 | Registrierung |
| `Frontend/ACM_Frontend/src/components/AddEndpointModal.jsx` | 71 | Endpoint hinzufügen |
| `Frontend/ACM_Frontend/src/components/SetIntervallModal.jsx` | 60 | Intervall setzen |
| `Frontend/ACM_Frontend/src/components/DeleteConfirmModal.jsx` | 19 | Löschbestätigung |
| `Frontend/ACM_Frontend/src/components/AccountSettingsModal.jsx` | 92 | Passwort/Email ändern |
| `Frontend/ACM_Frontend/src/components/LogModal.jsx` | 70 | Logs anzeigen/filtern |
| `Frontend/ACM_Frontend/src/components/EditUrlModal.jsx` | 40 | URL bearbeiten |
| `Frontend/ACM_Frontend/src/components/Sparkline.jsx` | 37 | Mini-Chart |
| `Frontend/ACM_Frontend/src/components/ThemeSwitcher.jsx` | 45 | Theme-Dropdown |

## Infrastruktur

| Datei | Zeilen | Zweck |
|-------|--------|-------|
| `docker-compose.yml` | 51 | 3 Services |
| `DB/createTables.sql` | 32 | 5 Tabellen |
| `setup.sh` | 51 | Ein-Klick-Deployment |
| `.env` | 10 | Konfiguration |
| `.gitignore` | 46 | Ignorierte Dateien |

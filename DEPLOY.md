# ACM API Connection Monitor — Deployment

## Schnellstart (Lokale Entwicklung)

**Voraussetzungen:** Rust, Node.js 22+, PostgreSQL 17 (lokal)

**1. `.env` konfigurieren**

Im Projekt-Root liegt `.env` – dort ggf. `DATABASE_URL`, `BACKEND_PORT`, `FRONTEND_PORT` etc. anpassen.

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

**4. Im Browser öffnen**

```bash
open http://localhost:8080
```

## Deployment auf AWS

### 1. RDS PostgreSQL einrichten

Über AWS Console oder CLI eine RDS-PostgreSQL-Instanz erstellen:

- **Engine:** PostgreSQL 17
- **Instance:** db.t3.micro (2 vCPU, 1 GB RAM) — für Produktion db.t3.medium
- **Storage:** 20 GB gp3, automatisches Scaling deaktiviert
- **VPC:** Standard-VPC (gleiche VPC wie EC2-Instanzen)
- **Public Access:** Nein (nur intern via Security Group)
- **Security Group:** Erlaube PostgreSQL (Port 5432) von den Security-Groups der Backend-EC2
- **DB-Name:** `database-acm`
- **Master-Username:** `acm_admin`
- **Master-Password:** starkes Passwort via Secrets Manager oder Parameter Store

Nach dem Erstellen die RDS-Endpoint-URL notieren (z. B. `database-acm.xxxxxxx.eu-central-1.rds.amazonaws.com`).

### 2. EC2 — Backend

#### Instanz starten

- **AMI:** Amazon Linux 2023
- **Typ:** t3.medium (2 vCPU, 4 GB RAM)
- **Security Group:** Port 3000 (API) von der Frontend-EC2-Security-Group erlauben, Port 22 (SSH) von deiner IP
- **Storage:** 20 GB gp3

#### Abhängigkeiten & Setup

```bash
sudo yum update -y
sudo yum install -y git

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

#### App deployen

```bash
git clone <dein-repo-url> /opt/acm-backend
cd /opt/acm-backend/Backend

# DATABASE_URL auf RDS-Endpoint setzen
export DATABASE_URL="postgres://acm_admin:<passwort>@database-acm.xxxxxxx.eu-central-1.rds.amazonaws.com:5432/database-acm"
export RUST_LOG=info

cargo build --release
./target/release/Backend
```

Für Produktion als Systemd-Service einrichten:

```ini
# /etc/systemd/system/acm-backend.service
[Unit]
Description=ACM Backend
After=network.target

[Service]
Type=simple
User=ec2-user
WorkingDirectory=/opt/acm-backend/Backend
EnvironmentFile=/opt/acm-backend/.env          # ← Läd DATABASE_URL, BACKEND_HOST, BACKEND_PORT aus .env
Environment=RUST_LOG=info
ExecStart=/opt/acm-backend/Backend/target/release/Backend
Restart=always

[Install]
WantedBy=multi-user.target
```

**Hinweis:** `EnvironmentFile=/opt/acm-backend/.env` lädt die `.env`-Datei aus dem
Repo-Root. Dort muss vor dem ersten Start `DATABASE_URL` auf den RDS-Endpoint
gesetzt sein (nicht `localhost:5432`). `systemd` setzt die Env-Vars → `dotenv::ok()`
überschreibt sie nicht.

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now acm-backend
```

### 3. EC2 — Frontend

#### Instanz starten

- **AMI:** Amazon Linux 2023
- **Typ:** t3.small (2 vCPU, 2 GB RAM)
- **Security Group:** Port 80 (HTTP) und 443 (HTTPS) von überall (`0.0.0.0/0`), Port 22 (SSH) von deiner IP
- **Storage:** 15 gp3
- **Elastic IP:** zuweisen (damit Domain darauf zeigt)

#### Abhängigkeiten & Setup

```bash
sudo yum update -y
sudo yum install -y git nginx

# Node.js 22
curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
sudo yum install -y nodejs
```

#### App bauen & deployen

```bash
git clone <dein-repo-url> /opt/acm-frontend
cd /opt/acm-frontend/Frontend

# VITE_API_URL NICHT setzen – die API läuft über denselben Nginx
# (Proxy /acm → Backend, siehe nginx-Konfiguration unten)
# echo "VITE_API_URL=http://..." > .env   # <-- NICHT nötig

npm ci
npm run build
```

#### Nginx-Konfiguration

Der Nginx serviert nicht nur die statischen Frontend-Dateien, sondern leitet auch
alle `/acm/*`-API-Anfragen an das Backend weiter (Reverse Proxy). Dadurch entfällt
das CORS-Problem und der Client-Browser muss keine privaten IPs erreichen.

```nginx
# /etc/nginx/conf.d/acm.conf
server {
    listen 80;
    server_name _;

    root /opt/acm-frontend/Frontend/dist;
    index index.html;

    # ─── API-Proxy ───
    # Alle Anfragen an /acm/* werden an das Backend weitergeleitet
    location /acm/ {
        proxy_pass http://<backend-private-ip>:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    # ─── Statische Frontend-Dateien ───
    # Alle anderen Anfragen liefern die SPA aus (Client-side Routing)
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

```bash
sudo systemctl enable --now nginx
```

Optional: HTTPS via Let's Encrypt/Certbot einrichten.

### 4. Domain (optional)

```bash
# Elastic IP der Frontend-EC2 zuweisen (AWS Console → EC2 → Elastic IPs)
# DNS A-Record auf die Elastic IP setzen
```

## Architektur

```
┌──────────┐       ┌──────────────────────┐       ┌──────────────┐       ┌─────────────┐
│  Browser │ ──►   │  Frontend-EC2        │ ──►   │   Backend    │ ──►   │  RDS (AWS)  │
│  :80     │       │  Nginx (Proxy :80)   │       │  EC2 / Axum  │       │  PostgreSQL │
└──────────┘       │  / :80 (static)      │       │  :3000       │       │  :5432      │
                   │  /acm/* → Backend    │       └──────────────┘       └─────────────┘
                   └──────────────────────┘
```

- **Frontend:** React SPA → Nginx auf eigener EC2 (t3.small) – serviert statische Dateien + proxyed `/acm/*` ans Backend
- **Backend:** Rust/Axum API auf eigener EC2 (t3.medium) – nur intern via Nginx-Proxy erreichbar
- **Datenbank:** PostgreSQL 17 via AWS RDS (db.t3.micro)
- **Monitoring:** Hintergrund-Task checkt alle aktiven Endpoints (HTTP / TCP / ICMP)
- **Kommunikation:** Browser → Frontend-EC2 (public), Frontend-EC2 → Backend-EC2 (intern via Nginx Proxy), Backend-EC2 → RDS (intern)

## Umgebungsvariablen (.env)

| Variable | Beschreibung | Beispiel |
|---|---|---|
| `DATABASE_URL` | Connection-String zum RDS | `postgres://acm_admin:pass@database-acm.xxx.rds.amazonaws.com:5432/database-acm` |
| `BACKEND_HOST` | Backend-Bind-Addresse | `0.0.0.0` |
| `BACKEND_PORT` | Backend-Port | `3000` |
| `FRONTEND_PORT` | Vite-Dev-Server-Port | `8080` |
| `API_PROXY_TARGET` | Vite-Proxy-Ziel (Backend-URL) | `http://localhost:3000` |
| `VITE_API_URL` | API-Basis-URL im Frontend-Code | `/acm` |
| `RUST_LOG` | Log-Level (Backend) | `info` |

> **Hinweis:** In Produktion wird `VITE_API_URL` nicht gesetzt (Default `/acm`), und Nginx proxyt `/acm/*` an das Backend.

## Session-Token (In-Memory)

Die Session-Tokens werden im **Arbeitsspeicher** des Backends gehalten
(HashMap in AppState, kein Redis/DB). Konsequenzen:

- **Backend-Neustart** → alle Token verloren → alle User müssen sich einmal neu einloggen (Daten bleiben erhalten)
- **Horizontales Scaling (mehrere Backend-Instanzen)** → nicht kompatibel (Token nur auf der Instanz gültig, auf der sie ausgestellt wurden)



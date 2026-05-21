# ACM API Connection Monitor — Deployment

## Schnellstart (Lokale Entwicklung)

**Voraussetzungen:** Rust, Node.js 22+, PostgreSQL 17 (lokal)

```bash
# Backend starten
cd Backend/API
DATABASE_URL=postgres://admin:admin@localhost:5432/mydb cargo run

# Frontend starten (zweites Terminal)
cd Frontend/ACM_Frontend
npm ci
npm run dev

# Im Browser öffnen
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
- **DB-Name:** `acmdb`
- **Master-Username:** `acm_admin`
- **Master-Password:** starkes Passwort via Secrets Manager oder Parameter Store

Nach dem Erstellen die RDS-Endpoint-URL notieren (z. B. `acmdb.xxxxxxx.eu-central-1.rds.amazonaws.com`).

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
cd /opt/acm-backend/Backend/API

# DATABASE_URL auf RDS-Endpoint setzen
export DATABASE_URL="postgres://acm_admin:<passwort>@acmdb.xxxxxxx.eu-central-1.rds.amazonaws.com:5432/acmdb"
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
WorkingDirectory=/opt/acm-backend/Backend/API
Environment=DATABASE_URL=postgres://acm_admin:<passwort>@acmdb.xxxxxxx.eu-central-1.rds.amazonaws.com:5432/acmdb
Environment=RUST_LOG=info
ExecStart=/opt/acm-backend/Backend/API/target/release/Backend
Restart=always

[Install]
WantedBy=multi-user.target
```

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
cd /opt/acm-frontend/Frontend/ACM_Frontend

# API-URL auf Backend-EC2 zeigen
echo "VITE_API_URL=http://<backend-private-ip>:3000" > .env

npm ci
npm run build
```

#### Nginx-Konfiguration

```nginx
# /etc/nginx/conf.d/acm.conf
server {
    listen 80;
    server_name _;

    root /opt/acm-frontend/Frontend/ACM_Frontend/dist;
    index index.html;

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
┌──────────┐       ┌──────────────┐       ┌──────────────┐       ┌─────────────┐
│  Browser │ ──►   │  Frontend    │ ──►   │   Backend    │ ──►   │  RDS (AWS)  │
│  :80     │       │  EC2 / Nginx │       │  EC2 / Axum  │       │  PostgreSQL │
└──────────┘       │  :80         │       │  :3000       │       │  :5432      │
                   └──────────────┘       └──────────────┘       └─────────────┘
```

- **Frontend:** React SPA → Nginx auf eigener EC2 (t3.small)
- **Backend:** Rust/Axum API auf eigener EC2 (t3.medium)
- **Datenbank:** PostgreSQL 17 via AWS RDS (db.t3.micro)
- **Monitoring:** Hintergrund-Task pingt alle aktiven Endpoints
- **Kommunikation:** Frontend-EC2 → Backend-EC2 (intern), Backend-EC2 → RDS (intern)

## Umgebungsvariablen (.env)

| Variable | Beschreibung | Beispiel |
|---|---|---|
| `DATABASE_URL` | Connection-String zum RDS | `postgres://acm_admin:pass@acmdb.xxx.rds.amazonaws.com:5432/acmdb` |
| `VITE_API_URL` | API-Basis-URL (Frontend) | `http://<backend-ip>:3000` |
| `RUST_LOG` | Log-Level (Backend) | `info` |



# ACM API Connection Monitor — Deployment

## Schnellstart (Lokale Entwicklung)

**Voraussetzungen:** Rust, Node.js 22+, PostgreSQL 17 (lokal)

**1. `.env` konfigurieren**

Zwei `.env`-Dateien:
- **`/.env`** (Projekt-Root) → `DATABASE_URL`, `BACKEND_PORT` etc. fürs Backend
- **`Frontend/.env`** → `VITE_API_URL` für Vite-Build (wichtig für Production!)

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

## Deployment auf AWS — Architektur

```
Browser ──HTTP──→ S3 Static Website ──HTTP──→ EC2 (Port 3000) ──intern──→ RDS PostgreSQL
                     (Frontend)                 (Backend + optional Nginx)
```

- **Frontend:** React SPA → gebaut mit Vite → gehostet auf **S3 Static Website** (HTTP)
- **Backend:** Rust/Axum API auf einer **EC2-Instanz** (Port 3000, direkt oder via Nginx)
- **Datenbank:** PostgreSQL 17 via AWS RDS
- **CORS:** Backend erlaubt explizit `Authorization`, `Content-Type`, `Accept`-Header

### Wichtig: EC2 Public IP

EC2-Public-IP wechselt bei jedem **Stop/Start**. Lösung:
- **Nur Reboot** verwenden (Reboot behält die IP)
- Nach Stop/Start: `Frontend/.env` mit neuer IP updaten, Frontend neu bauen + nach S3 deployen
- Elastic IP ist empfohlen, aber im Schul-Account oft nicht erlaubt

---

### 1. RDS PostgreSQL einrichten

Über AWS Console oder CLI eine RDS-PostgreSQL-Instanz erstellen:

- **Engine:** PostgreSQL 17
- **Instance:** db.t3.micro (2 vCPU, 1 GB RAM) — für Produktion db.t3.medium
- **Storage:** 20 GB gp3, automatisches Scaling deaktiviert
- **VPC:** Standard-VPC (gleiche VPC wie EC2-Instanz)
- **Public Access:** Nein (nur intern via Security Group)
- **Security Group:** Erlaube PostgreSQL (Port 5432) von der Backend-EC2-Security-Group
- **DB-Name:** `database-acm`
- **Master-Username:** `postgres`
- **Master-Password:** starkes Passwort via Secrets Manager oder Parameter Store

Nach dem Erstellen die RDS-Endpoint-URL notieren (z. B. `database-acm.xxxxxxx.eu-west-1.rds.amazonaws.com`).

---

### 2. EC2 — Backend

#### Instanz starten

- **AMI:** Amazon Linux 2023
- **Typ:** t3.medium (2 vCPU, 4 GB RAM)
- **Security Group:** Port 3000 (API) und Port 22 (SSH) von überall (`0.0.0.0/0`)
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
git clone <dein-repo-url> /home/ec2-user/ACM_API-Connection-Monitor
cd /home/ec2-user/ACM_API-Connection-Monitor/Backend

# DATABASE_URL auf RDS-Endpoint setzen
export DATABASE_URL="postgres://acm_admin:<passwort>@database-acm.xxxxxxx.eu-west-1.rds.amazonaws.com:5432/database-acm"
export RUST_LOG=info

cargo build --release
./target/release/Backend
```

#### Systemd-Service (Production)

```ini
# /etc/systemd/system/acm-backend.service
[Unit]
Description=ACM Backend
After=network.target

[Service]
Type=simple
User=ec2-user
WorkingDirectory=/home/ec2-user/ACM_API-Connection-Monitor/Backend
EnvironmentFile=/home/ec2-user/ACM_API-Connection-Monitor/.env
Environment=RUST_LOG=info
ExecStart=/home/ec2-user/ACM_API-Connection-Monitor/Backend/target/release/Backend
Restart=always

[Install]
WantedBy=multi-user.target
```

**.env auf der EC2** muss `DATABASE_URL` auf den RDS-Endpoint zeigen (nicht localhost):
```env
DATABASE_URL=postgres://acm_admin:pass@database-acm.xxxxxxx.eu-west-1.rds.amazonaws.com:5432/database-acm
BACKEND_HOST=0.0.0.0
BACKEND_PORT=3000
RUST_LOG=info
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now acm-backend
sudo journalctl -u acm-backend -f   # Logs live verfolgen
```

#### Backend neustarten (nach Code-Änderungen)

```bash
ssh ec2-user@<EC2-IP>
cd ACM_API-Connection-Monitor && git pull
cd Backend && cargo build --release
sudo systemctl restart acm-backend
```

---

### 3. S3 Static Website — Frontend

#### Bucket erstellen

1. **AWS Console → S3 → Bucket erstellen**
   - Name: z.B. `acm-fe-bucket`
   - Region: `eu-west-1` (gleiche Region wie EC2/RDS)
   - **Block Public Access deaktivieren** (für Static Website nötig)

2. **Static Website Hosting aktivieren:**
   - Bucket → **Properties** → **Static website hosting** → **Enable**
   - Index document: `index.html`
   - Error document: `index.html` (wichtig für SPA-Routing!)
   - Bucket-URL notieren: `http://acm-fe-bucket.s3-website-eu-west-1.amazonaws.com`

3. **Bucket Policy (Public Read):**
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Sid": "PublicReadGetObject",
         "Effect": "Allow",
         "Principal": "*",
         "Action": "s3:GetObject",
         "Resource": "arn:aws:s3:::acm-fe-bucket/*"
       }
     ]
   }
   ```
   Falls der Schul-Account keine Public Policies erlaubt: mit `--acl public-read` hochladen.

#### Frontend lokal bauen & deployen

**`Frontend/.env`** setzen (Vite liest nur diese Datei!):
```env
VITE_API_URL=http://<AKTUELLE-EC2-IP>:3000/acm
```
> **Wichtig:** Bei EC2-Stop/Start ändert sich die IP → `Frontend/.env` updaten + neu bauen + deployen.

```bash
cd Frontend
npm ci
npm run build
aws s3 sync dist/ s3://acm-fe-bucket/ --delete --acl public-read
```

Danach im Browser öffnen: `http://acm-fe-bucket.s3-website-eu-west-1.amazonaws.com`

---

### 4. Nginx auf EC2 (optional, für Reverse-Proxy)

Nginx kann auf der EC2 als Reverse-Proxy vor dem Backend laufen:

```bash
sudo yum install -y nginx
```

```nginx
# /etc/nginx/conf.d/acm.conf
server {
    listen 80;
    server_name _;

    location /acm/ {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    location / {
        return 404;
    }
}
```

```bash
sudo systemctl enable --now nginx
```

Dann `VITE_API_URL=http://<EC2-IP>/acm` (ohne Port 3000) setzen.

---

## Umgebungsvariablen

### Root-`/.env` (für Backend / systemd)

| Variable | Beschreibung | Beispiel |
|---|---|---|
| `DATABASE_URL` | Connection-String zum RDS | `postgres://acm_admin:pass@database-acm.xxx.rds.amazonaws.com:5432/database-acm` |
| `BACKEND_HOST` | Backend-Bind-Addresse | `0.0.0.0` |
| `BACKEND_PORT` | Backend-Port | `3000` |
| `RUST_LOG` | Log-Level (Backend) | `info` |

### `Frontend/.env` (für Vite-Build)

| Variable | Beschreibung | Beispiel |
|---|---|---|
| `VITE_API_URL` | API-Basis-URL (in Production: EC2-IP) | `http://18.192.100.50:3000/acm` |
| `FRONTEND_PORT` | Vite-Dev-Server-Port | `8080` |
| `API_PROXY_TARGET` | Vite-Proxy-Ziel (Dev) | `http://localhost:3000` |

> **Hinweis:** `VITE_API_URL` wird zur Build-Zeit ins JS-Bundle eingebrannt. Nach jeder Änderung muss das Frontend neu gebaut + deployed werden.

---

## Session-Token (In-Memory)

Die Session-Tokens werden im **Arbeitsspeicher** des Backends gehalten
(HashMap in AppState, kein Redis/DB). Konsequenzen:

- **Backend-Neustart** → alle Token verloren → alle User müssen sich einmal neu einloggen (Daten bleiben erhalten)
- **Horizontales Scaling (mehrere Backend-Instanzen)** → nicht kompatibel (Token nur auf der Instanz gültig, auf der sie ausgestellt wurden)

---

## Troubleshooting

| Problem | Ursache | Lösung |
|---|---|---|
| CORS: Authorization blocked | `allow_headers(Any)` → Browser ignoriert `*` | Explizite Header setzen: `allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])` |
| Mixed Content: HTTPS page loads HTTP API | S3 REST-Endpoint (HTTPS) + API (HTTP) | S3 Static Website (HTTP) verwenden statt REST-URL |
| Fetch failed / Status (null) | EC2-IP geändert oder Security Group blockiert | IP updaten, Security Group prüfen |
| 403 favicon.svg | Datei fehlt in S3 | Ignorieren oder `public/favicon.svg` ins Repo legen |

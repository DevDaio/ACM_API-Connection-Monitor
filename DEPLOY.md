# ACM API Connection Monitor — Deployment

## Schnellstart (Lokale Entwicklung)

**Voraussetzungen:** Rust, Node.js 22+, PostgreSQL 17 (lokal)

**1. `.env` konfigurieren**

Zwei `.env`-Dateien:
- **`/.env`** (Projekt-Root) → `DATABASE_URL`, `BACKEND_PORT` etc. fürs Backend
- **`Frontend/.env`** → `VITE_API_URL` für Vite-Build (in Production leer lassen)

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

---

## Deployment auf AWS — Architektur

```
Browser ──HTTPS──→ CloudFront
                      │
                      ├── /*       → S3 Static Website (Custom Origin)
                      │
                      └── /acm/*   → EC2 Nginx:80 → proxy → localhost:3000
                                                              │
                                                              └── RDS PostgreSQL
```

- **Frontend:** React SPA → gebaut mit Vite → gehostet auf **S3 Static Website** → ausgeliefert via **CloudFront** (HTTPS)
- **Backend:** Rust/Axum API auf **EC2** (Port 3000) hinter **Nginx Reverse-Proxy** (Port 80)
- **Datenbank:** PostgreSQL 17 via **AWS RDS** (nur intern erreichbar)
- **CloudFront:** Einheitlicher HTTPS-Endpunkt, routet `/*` → S3 und `/acm/*` → EC2
- **CORS:** Nicht mehr nötig – Frontend und Backend laufen unter derselben CloudFront-Domain

### Elastic IP

Die EC2-Instanz braucht eine **Elastic IP** (feste öffentliche IP), damit der CloudFront-Origin
stabil bleibt. Elastic IPs sind kostenlos, solange sie einer laufenden Instanz zugeordnet sind.

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

### 2. Elastic IP

1. **AWS Console → EC2 → Elastic IPs → Elastic IP-Adresse zuweisen**
2. **Region:** `eu-west-1` (gleiche Region wie EC2)
3. Die Elastic IP der EC2-Instanz zuordnen
4. Die IP notieren (wird für CloudFront-Origin und nginx gebraucht)

---

### 3. EC2 — Backend + Nginx

#### Instanz starten

- **AMI:** Amazon Linux 2023
- **Typ:** t3.medium (2 vCPU, 4 GB RAM)
- **Security Group:**
  - Port 22 (SSH) von deiner IP (`/32`)
  - Port 80 (HTTP) von `0.0.0.0/0` (CloudFront greift auf Port 80 zu)
  - Port 3000 nur für internen Zugriff (nginx → localhost)
- **Storage:** 20 GB gp3

#### Abhängigkeiten & Setup

```bash
sudo dnf update -y
sudo dnf install -y git nginx

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
```

#### Nginx Reverse-Proxy konfigurieren

```bash
# Config aus dem Repo kopieren
sudo cp /home/ec2-user/ACM_API-Connection-Monitor/EC2/nginx/acm-backend.conf /etc/nginx/conf.d/

# Syntax prüfen und starten
sudo nginx -t
sudo systemctl enable --now nginx
```

Der nginx lauscht auf Port 80 und leitet `/acm/*` an `http://127.0.0.1:3000` weiter.

#### Systemd-Service (Production)

```ini
# /etc/systemd/system/acm-backend.service
[Unit]
Description=ACM Backend
After=network.target nginx.service

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

**`.env` auf der EC2**:
```env
DATABASE_URL=postgres://acm_admin:pass@database-acm.xxxxxxx.eu-west-1.rds.amazonaws.com:5432/database-acm
BACKEND_HOST=127.0.0.1
BACKEND_PORT=3000
RUST_LOG=info
```

> **Wichtig:** `BACKEND_HOST=127.0.0.1` – der Rust-Server bindet nur localhost,
> damit nur nginx (und nicht direkt das Internet) darauf zugreifen kann.

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now acm-backend
sudo journalctl -u acm-backend -f   # Logs live verfolgen
```

#### Backend neustarten (nach Code-Änderungen)

```bash
ssh ec2-user@<ELASTIC-IP>
cd ACM_API-Connection-Monitor && git pull
cd Backend && cargo build --release
sudo systemctl restart acm-backend
```

---

### 4. S3 Static Website — Frontend

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

#### Frontend lokal bauen & deployen

**`Frontend/.env`**: `VITE_API_URL` leer lassen oder auskommentieren →
der Frontend-Code verwendet den relativen Pfad `/acm/...`.
CloudFront routet `/acm/*` automatisch zum Backend.

```bash
cd Frontend
npm ci
npm run build
aws s3 sync dist/ s3://acm-fe-bucket/ --delete --acl public-read
```

---

### 5. CloudFront Distribution

Eine CloudFront-Distribution bündelt S3 (Frontend) und EC2 (Backend)
unter einer gemeinsamen HTTPS-Domain.

#### Distribution erstellen

**AWS Console → CloudFront → Distribution erstellen**

**Origin 1 — S3 (Frontend):**
| Einstellung | Wert |
|---|---|
| Origin Domain | `acm-fe-bucket.s3-website-eu-west-1.amazonaws.com` |
| Protocol | HTTP only (S3 Static Website spricht nur HTTP) |
| Origin Path | leer lassen |

> **Hinweis:** Wir verwenden den **S3 Website-Endpoint** (nicht den REST-Endpoint),
> weil Static Website Hosting aktiviert ist. Dementsprechend wird der Origin als
> **Custom Origin** konfiguriert, nicht als S3 Origin.

**Origin 2 — EC2 (Backend):**
| Einstellung | Wert |
|---|---|
| Origin Domain | `<ELASTIC-IP>` (z.B. `18.192.100.50`) |
| Protocol | HTTP only |
| Origin Path | leer lassen |

**Default Behavior (`*` → S3):**
| Einstellung | Wert |
|---|---|
| Origin | S3-Origin |
| Viewer Protocol Policy | **Redirect HTTP → HTTPS** |
| Allowed HTTP Methods | GET, HEAD, OPTIONS |
| Cache Policy | `CachingOptimized` (empfohlen) |

**Behavior (`/acm*` → EC2):**
| Einstellung | Wert |
|---|---|
| Origin | EC2-Origin |
| Viewer Protocol Policy | **Redirect HTTP → HTTPS** |
| Allowed HTTP Methods | GET, HEAD, OPTIONS, PUT, POST, DELETE |
| Cache Policy | **`CachingDisabled`** (API-Responses nicht cachen) |
| Query Strings | **Forward all, cache based on all** |
| Headers | **Forward `Authorization`, `Content-Type`** |

#### Nach dem Erstellen

- **Distribution-Domain notieren:** `https://dxxxxxxxxxxxxx.cloudfront.net`
- **Status abwarten:** Bis `Last Modified` nicht mehr `InProgress` zeigt
  (ca. 5–10 Minuten)

#### S3 im Browser öffnen

`https://dxxxxxxxxxxxxx.cloudfront.net` → Frontend wird geladen.
API-Calls gehen als **same-origin** über CloudFront → EC2.

---

### 6. Umgebungsvariablen

#### Root-`/.env` (für Backend / systemd)

| Variable | Beschreibung | Beispiel |
|---|---|---|
| `DATABASE_URL` | Connection-String zum RDS | `postgres://acm_admin:pass@database-acm.xxx.rds.amazonaws.com:5432/database-acm` |
| `BACKEND_HOST` | Backend-Bind-Addresse | `127.0.0.1` (nur localhost) |
| `BACKEND_PORT` | Backend-Port | `3000` |
| `RUST_LOG` | Log-Level (Backend) | `info` |

#### `Frontend/.env` (für Vite-Build)

| Variable | Beschreibung | Beispiel |
|---|---|---|
| `VITE_API_URL` | leer lassen → relativer Pfad `/acm/...` (CloudFront routet) | *(nicht setzen)* |
| `FRONTEND_PORT` | Vite-Dev-Server-Port | `8080` |
| `API_PROXY_TARGET` | Vite-Proxy-Ziel (Dev) | `http://localhost:3000` |

> **Hinweis:** `VITE_API_URL` wird zur Build-Zeit ins JS-Bundle eingebrannt.
> In Production leer lassen, damit `BASE = '/acm'` genutzt wird.

---

### 7. Nützliche Befehle

```bash
# Backend-Logs live
sudo journalctl -u acm-backend -f

# Nginx-Status
sudo systemctl status nginx
sudo nginx -t

# Nginx-Logs
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log

# Elastic IP prüfen
curl -s http://<ELASTIC-IP>/acm

# CloudFront Invalidierung (bei Frontend-Update)
aws cloudfront create-invalidation --distribution-id <DIST-ID> --paths "/*"
```

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
| CloudFront: 403 Access Denied | S3-Bucket-Policy fehlt oder falsch | Bucket-Policy mit `PublicReadGetObject` prüfen |
| CloudFront: 502 Bad Gateway (EC2) | EC2 nicht erreichbar oder nginx läuft nicht | Elastic IP prüfen, `sudo systemctl status nginx` |
| 504 Gateway Timeout | Backend antwortet nicht | `sudo journalctl -u acm-backend -f` prüfen |
| Fetch failed / Status (null) | CloudFront-Distribution noch im Deployment | Warten bis Status "Deployed" zeigt (5–10 Min) |
| Frontend lädt, API-Calls schlagen fehl | CloudFront-Verhalten `/acm*` falsch konfiguriert | Behavior `/acm*` prüfen: Query Strings + Headers forwarden |
| 403 favicon.svg / page refresh 404 | SPA-Routing fehlt | Error document in S3 Static Website auf `index.html` setzen |
| Backend-Neustart: alle ausgeloggt | In-Memory-Sessions | Neu einloggen – Daten bleiben erhalten |

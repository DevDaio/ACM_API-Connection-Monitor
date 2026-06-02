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
Browser ──HTTP──→ EC2 Nginx:80
                      │
                      ├── /        → reverse proxy → S3 Static Website (Frontend)
                      │
                      └── /acm/*   → proxy → localhost:3000 (Rust/Axum Backend)
                                                  │
                                                  └── RDS PostgreSQL
```

- **Frontend:** React SPA → gebaut mit Vite → gehostet auf **S3 Static Website** → ausgeliefert via **Nginx Reverse-Proxy** auf EC2
- **Backend:** Rust/Axum API auf **EC2** (Port 3000), ebenfalls hinter Nginx (Port 80)
- **Datenbank:** PostgreSQL 17 via **AWS RDS** (nur intern erreichbar)
- **Nginx:** Einheitlicher Entrypoint auf Port 80 → routet `/` → S3 und `/acm/*` → Backend
- **CORS:** Nicht nötig – Frontend und Backend laufen unter derselben EC2-IP (same-origin)

### EC2 Public IP (dynamisch)

Die EC2-Public-IP wechselt bei **Stop/Start**. Lösung:
- Nur **Reboot** verwenden (Reboot behält die IP)
- Nach IP-Wechsel: nginx-Konfiguration muss nicht geändert werden, aber die URL zum Aufrufen ändert sich

---

## HTTPS aufsetzen (optional)

Aktuell läuft alles über **HTTP** (Port 80). Für HTTPS gibt es zwei Wege:

### Variante A: Self-Signed Certificate (einfach)

```bash
# Self-Signed Cert erstellen
sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/acm-selfsigned.key \
  -out /etc/ssl/certs/acm-selfsigned.crt \
  -subj "/CN=$(curl -s http://checkip.amazonaws.com)"

# Nginx-Konfiguration: Port 443 + Redirect von 80
sudo nano /etc/nginx/conf.d/acm-backend.conf
```

**`/etc/nginx/conf.d/acm-backend.conf`**:
```nginx
server {
    listen 80;
    server_name _;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name _;

    ssl_certificate     /etc/ssl/certs/acm-selfsigned.crt;
    ssl_certificate_key /etc/ssl/private/acm-selfsigned.key;

    location / {
        proxy_pass http://acm-fe-bucket.s3-website-eu-west-1.amazonaws.com;
        proxy_set_header Host acm-fe-bucket.s3-website-eu-west-1.amazonaws.com;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
    }

    location /acm/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
    }
}
```

```bash
sudo nginx -t && sudo systemctl reload nginx
```

**Browser zeigt einmal "Unsicher"** – einmalig "Trotzdem fortfahren" klicken.
Bei IP-Wechsel muss das Zertifikat neu generiert werden.

### Variante B: Domain + Let's Encrypt (empfohlen)

Wenn du eine Domain (z.B. `acm-api.example.com`) hast, die auf die EC2-IP zeigt:

```bash
sudo dnf install -y certbot python3-certbot-nginx
sudo certbot --nginx -d acm-api.example.com
# Automatische Verlängerung
sudo certbot renew --dry-run
```

Dann läuft alles sauber über HTTPS ohne Browser-Warnung.

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

### 2. EC2 — Backend + Nginx

#### Instanz starten

- **AMI:** Amazon Linux 2023
- **Typ:** t3.medium (2 vCPU, 4 GB RAM)
- **Security Group:**
  - Port 22 (SSH) von deiner IP (`/32`)
  - Port 80 (HTTP) von `0.0.0.0/0` (Nginx als Frontend + API-Proxy)
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

Der nginx lauscht auf Port 80 und leitet:
- `/` → S3 Static Website (Frontend)
- `/acm/*` → `http://127.0.0.1:3000` (Backend)

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
ssh ec2-user@<EC2-PUBLIC-IP>
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

#### Frontend lokal bauen & deployen

**`Frontend/.env`**: `VITE_API_URL` leer lassen →
der Frontend-Code verwendet den relativen Pfad `/acm/...`.
Nginx auf der EC2 proxyt `/acm/*` automatisch zum Backend.

```bash
cd Frontend
npm ci
npm run build
aws s3 sync dist/ s3://acm-fe-bucket/ --delete --acl public-read
```

---

### 4. App im Browser öffnen

`http://<EC2-PUBLIC-IP>` → Nginx serviert das Frontend aus S3.
API-Calls gehen als **same-origin** über Nginx → localhost:3000.

> **Hinweis:** Die S3-Website-URL (`http://acm-fe-bucket.s3-website-...`) wird nicht mehr direkt aufgerufen – alles läuft über die EC2-IP.

---

### 5. Umgebungsvariablen

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
| `VITE_API_URL` | leer lassen → relativer Pfad `/acm/...` (Nginx proxyt) | *(nicht setzen)* |
| `FRONTEND_PORT` | Vite-Dev-Server-Port | `8080` |
| `API_PROXY_TARGET` | Vite-Proxy-Ziel (Dev) | `http://localhost:3000` |

> **Hinweis:** `VITE_API_URL` wird zur Build-Zeit ins JS-Bundle eingebrannt.
> In Production leer lassen, damit `BASE = '/acm'` genutzt wird.

---

### 6. Nützliche Befehle

```bash
# Backend-Logs live
sudo journalctl -u acm-backend -f

# Nginx-Status
sudo systemctl status nginx
sudo nginx -t

# Nginx-Logs
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log

# Backend direkt testen
curl -s http://127.0.0.1:3000/acm
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
| 502 Bad Gateway | EC2-IP geändert (Stop/Start) | Neue IP im Browser eingeben – nginx läuft weiter |
| Connection refused | Rust-Backend läuft nicht | `sudo journalctl -u acm-backend -f` prüfen |
| Frontend lädt, API tot | Nginx `/acm/`-Proxy falsch | `sudo tail -f /var/log/nginx/error.log` |
| 403 favicon.svg / blank page | SPA-Routing falsch | Error document in S3 auf `index.html` prüfen |
| Backend-Neustart: alle ausgeloggt | In-Memory-Sessions | Neu einloggen – Daten bleiben erhalten |
| Zen Browser CORS-Fehler | Alte S3-URL im Lesezeichen | `http://<EC2-IP>` direkt verwenden (same-origin) |

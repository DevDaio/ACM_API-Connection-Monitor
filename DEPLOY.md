# ACM API Connection Monitor — Deployment

## Schnellstart (Lokale Entwicklung)

**Voraussetzungen:** Docker, Rust, Node.js 22+

```bash
# Einmaliger Setup
chmod +x setup.sh
./setup.sh

# Im Browser öffnen
open http://localhost
```

## Manuelle Schritte

```bash
# 1. Backend bauen
cd Backend/API
cargo build --release
docker build -t acm-backend:latest -f Dockerfile.local .
cd ../..

# 2. Frontend bauen
cd Frontend/ACM_Frontend
npm ci
npm run build
docker build -t acm-frontend:latest .
cd ../..

# 3. Starten
docker compose up -d

# 4. Logs
docker compose logs -f
```

## Deployment auf AWS (EC2)

### 1. EC2-Instanz starten

- **AMI:** Amazon Linux 2023
- **Typ:** t3.medium (2 vCPU, 4 GB RAM)
- **Security Group:** Port 80 (HTTP) und 22 (SSH) öffnen
- **Storage:** 20 GB gp3

### 2. Abhängigkeiten installieren

```bash
sudo yum update -y
sudo yum install -y docker git
sudo systemctl enable docker --now
sudo usermod -aG docker ec2-user

# Node.js 22
curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
sudo yum install -y nodejs

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

Aus- und wieder einloggen für Docker-Rechte.

### 3. App deployen

```bash
git clone <dein-repo-url> /opt/acm
cd /opt/acm
./setup.sh
```

### 4. Domain (optional)

```bash
# Elastic IP zuweisen (AWS Console → EC2 → Elastic IPs)
# DNS A-Record auf die Elastic IP setzen
```

## CI/CD (GitHub Actions)

Erstelle `.github/workflows/deploy.yml`:

```yaml
name: Deploy
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Backend bauen
        run: |
          cd Backend/API
          cargo build --release
          docker build -t acm-backend:latest -f Dockerfile.local .

      - name: Frontend bauen
        run: |
          cd Frontend/ACM_Frontend
          npm ci
          npm run build
          docker build -t acm-frontend:latest .

      - name: Auf Server kopieren & starten
        uses: appleboy/scp-action@v0.1.7
        with:
          host: ${{ secrets.HOST }}
          username: ec2-user
          key: ${{ secrets.SSH_KEY }}
          source: "./*"
          target: "/opt/acm"

      - name: Docker Compose starten
        uses: appleboy/ssh-action@v1.2.0
        with:
          host: ${{ secrets.HOST }}
          username: ec2-user
          key: ${{ secrets.SSH_KEY }}
          script: cd /opt/acm && docker compose up -d
```

## Umgebungsvariablen (.env)

| Variable | Beschreibung | Default |
|---|---|---|
| `POSTGRES_USER` | DB-Benutzer | `admin` |
| `POSTGRES_PASSWORD` | DB-Passwort | `admin` |
| `POSTGRES_DB` | DB-Name | `mydb` |
| `DATABASE_URL` | Connection-String (Backend) | `postgres://admin:admin@postgres:5432/mydb` |

## Architektur

```
┌──────────┐       ┌──────────┐       ┌────────────┐
│  Browser │ ──►   │  Nginx   │ ──►   │  Backend   │ ──►  PostgreSQL
│  :80     │       │  :80     │       │  :3000     │       :5432
└──────────┘       └──────────┘       └────────────┘
                        │
                        └──► Statische SPA (React)
```

- **Frontend:** React SPA, served via Nginx
- **Backend:** Rust/Axum API
- **Datenbank:** PostgreSQL 17
- **Monitoring:** Hintergrund-Task pingt alle aktiven Endpoints

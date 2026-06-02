#!/usr/bin/env bash
# Setup-Skript: Baut Backend und Frontend für lokale Entwicklung
set -euo pipefail

echo "╔══════════════════════════════════════════╗"
echo "║   ACM API Connection Monitor — Setup    ║"
echo "╚══════════════════════════════════════════╝"

# ─── Prüfen, ob benötigte Tools installiert sind ───
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo fehlt"; exit 1; }
command -v npm  >/dev/null 2>&1 || { echo "❌ Node.js/npm fehlt"; exit 1; }

# ─── .env interaktiv erstellen ───
if [ ! -f .env ]; then
  echo "→ Keine .env gefunden — lege interaktiv eine an …"
  echo ""

  read -r -p "POSTGRES_USER        [postgres]: " input
  POSTGRES_USER="${input:-postgres}"

  read -r -s -p "POSTGRES_PASSWORD    [admin123!]: " input
  echo
  POSTGRES_PASSWORD="${input:-admin123!}"

  read -r -p "POSTGRES_DB          [database-acm]: " input
  POSTGRES_DB="${input:-database-acm}"

  DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}"

  read -r -p "BACKEND_HOST         [0.0.0.0]: " input
  BACKEND_HOST="${input:-0.0.0.0}"

  read -r -p "BACKEND_PORT         [3000]: " input
  BACKEND_PORT="${input:-3000}"

  read -r -p "RUST_LOG             [info]: " input
  RUST_LOG="${input:-info}"

  # ─── Root-.env (Backend / systemd) ───
  cat > .env <<-EOF
# ─── ACM API Connection Monitor — Backend ───
# Wird via EnvironmentFile= von systemd geladen (EC2).
# Setze DATABASE_URL auf den RDS-Endpoint, nicht localhost!

POSTGRES_USER=${POSTGRES_USER}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
POSTGRES_DB=${POSTGRES_DB}

DATABASE_URL=${DATABASE_URL}

BACKEND_HOST=${BACKEND_HOST}
BACKEND_PORT=${BACKEND_PORT}

RUST_LOG=${RUST_LOG}
EOF

  echo "✅ .env (Backend) wurde erstellt"
  echo ""
fi

# ─── Frontend/.env (Vite-Build) ───
if [ ! -f Frontend/.env ]; then
  read -r -p "FRONTEND_PORT        [8080]: " input
  FRONTEND_PORT="${input:-8080}"

  read -r -p "VITE_API_URL (Dev)   [/acm]: " input
  VITE_API_URL="${input:-/acm}"

  cat > Frontend/.env <<-EOF
# ─── ACM API Connection Monitor — Frontend ───
# Wird von Vite zur Build-Zeit gelesen (import.meta.env).
# In Production: VITE_API_URL leer lassen → CloudFront routet /acm/* zum Backend.
# Elastic IP + CloudFront: IP-Wechsel ist kein Problem mehr.

FRONTEND_PORT=${FRONTEND_PORT}

API_PROXY_TARGET=http://localhost:${BACKEND_PORT:-3000}

VITE_API_URL=${VITE_API_URL}
EOF

  echo "✅ Frontend/.env (Frontend) wurde erstellt"
  echo ""
fi

# ─── Backend bauen ───
echo ""
echo "→ Backend bauen …"
cd Backend
cargo build --release  # Rust-Release-Build
cd ..
echo "✅ Backend gebaut"

# ─── Frontend bauen ───
echo ""
echo "→ Frontend bauen …"
cd Frontend
if [ -f package-lock.json ]; then
  npm ci                    # Installiert exakte Dependencies
else
  npm install               # Fallback, falls kein lockfile existiert
fi
npm run build             # Vite-Produktions-Build
cd ..
echo "✅ Frontend gebaut"

# ─── Erfolgsmeldung ───
echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   ✅  Build abgeschlossen!               ║"
echo "║                                          ║"
echo "║   Backend:   Backend/target/release/     ║"
echo "║   Frontend:  Frontend/dist/             ║"
echo "╚══════════════════════════════════════════╝"

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

  read -r -p "FRONTEND_PORT        [8080]: " input
  FRONTEND_PORT="${input:-8080}"

  API_PROXY_TARGET="http://localhost:${BACKEND_PORT}"

  read -r -p "VITE_API_URL         [/acm]: " input
  VITE_API_URL="${input:-/acm}"

  cat > .env <<-EOF
# ─── ACM API Connection Monitor ───
# Alle IPs/Ports/Verbindungen werden hier zentral konfiguriert.
# Das Backend liest zur Laufzeit, Vite zur Build-/Dev-Zeit.

POSTGRES_USER=${POSTGRES_USER}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
POSTGRES_DB=${POSTGRES_DB}

DATABASE_URL=${DATABASE_URL}

BACKEND_HOST=${BACKEND_HOST}
BACKEND_PORT=${BACKEND_PORT}

FRONTEND_PORT=${FRONTEND_PORT}
API_PROXY_TARGET=${API_PROXY_TARGET}

VITE_API_URL=${VITE_API_URL}
EOF

  echo "✅ .env wurde erstellt"
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

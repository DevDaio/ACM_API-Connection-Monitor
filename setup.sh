#!/usr/bin/env bash
set -euo pipefail

echo "╔══════════════════════════════════════════╗"
echo "║   ACM API Connection Monitor — Setup    ║"
echo "╚══════════════════════════════════════════╝"

# ─── Prüfen ───
command -v docker >/dev/null 2>&1 || { echo "❌ Docker fehlt"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo fehlt"; exit 1; }
command -v npm  >/dev/null 2>&1 || { echo "❌ Node.js/npm fehlt"; exit 1; }

# ─── .env ───
if [ ! -f .env ]; then
  cp .env.example .env 2>/dev/null || touch .env
  echo "✅ .env angelegt"
fi

# ─── Backend ───
echo ""
echo "→ Backend bauen …"
cd Backend/API
cargo build --release
cd ../..

docker build -t acm-backend:latest -f Backend/API/Dockerfile.local Backend/API/
echo "✅ Backend-Image gebaut"

# ─── Frontend ───
echo ""
echo "→ Frontend bauen …"
cd Frontend/ACM_Frontend
npm ci
npm run build
docker build -t acm-frontend:latest .
cd ../..
echo "✅ Frontend-Image gebaut"

# ─── Starten ───
echo ""
echo "→ Container starten …"
docker compose up -d

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   ✅  Deployment abgeschlossen!          ║"
echo "║                                          ║"
echo "║   Frontend:  http://localhost:8080          ║"
echo "║   Backend:   http://localhost:3000/acm    ║"
echo "║   Database:  localhost:5432               ║"
echo "╚══════════════════════════════════════════╝"

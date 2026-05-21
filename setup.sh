#!/usr/bin/env bash
# Setup-Skript: Baut Backend und Frontend für lokale Entwicklung
set -euo pipefail

echo "╔══════════════════════════════════════════╗"
echo "║   ACM API Connection Monitor — Setup    ║"
echo "╚══════════════════════════════════════════╝"

# ─── Pruefen, ob benoetigte Tools installiert sind ───
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo fehlt"; exit 1; }
command -v npm  >/dev/null 2>&1 || { echo "❌ Node.js/npm fehlt"; exit 1; }

# ─── .env anlegen, falls nicht vorhanden ───
if [ ! -f .env ]; then
  cp .env.example .env 2>/dev/null || touch .env
  echo "✅ .env angelegt"
fi

# ─── Backend bauen ───
echo ""
echo "→ Backend bauen …"
cd Backend/API
cargo build --release  # Rust-Release-Build
cd ../..
echo "✅ Backend gebaut"

# ─── Frontend bauen ───
echo ""
echo "→ Frontend bauen …"
cd Frontend/ACM_Frontend
npm ci                    # Installiert exakte Dependencies
npm run build             # Vite-Produktions-Build
cd ../..
echo "✅ Frontend gebaut"

# ─── Erfolgsmeldung ───
echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   ✅  Build abgeschlossen!               ║"
echo "║                                          ║"
echo "║   Backend:   Backend/API/target/release/ ║"
echo "║   Frontend:  Frontend/ACM_Frontend/dist/ ║"
echo "╚══════════════════════════════════════════╝"

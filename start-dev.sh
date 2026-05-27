#!/usr/bin/env bash
# Startet Backend + Frontend parallel für lokale Entwicklung.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "╔══════════════════════════════════════════╗"
echo "║   ACM API Connection Monitor — Dev      ║"
echo "╚══════════════════════════════════════════╝"

# ─── .env einlesen ───
if [ -f .env ]; then
  echo "→ Lade .env …"
  set -a
  # shellcheck source=/dev/null
  source .env
  set +a
else
  echo "⚠  Keine .env gefunden — verwende Defaults"
fi

# ─── Cleanup beim Beenden ───
cleanup() {
  echo ""
  echo "→ Beende Prozesse …"
  kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
  wait $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
  echo "✅ Alle Prozesse gestoppt"
}
trap cleanup SIGINT SIGTERM EXIT

# ─── Backend starten ───
echo "→ Backend starten (cargo run) …"
cd Backend
cargo run &
BACKEND_PID=$!
cd "$SCRIPT_DIR"

# ─── Frontend starten ───
echo "→ Frontend starten (npm run dev) …"
cd Frontend
npm run dev &
FRONTEND_PID=$!
cd "$SCRIPT_DIR"

echo ""
echo "⚡ Backend PID: $BACKEND_PID   Frontend PID: $FRONTEND_PID"
echo "   Drücke Ctrl+C zum Beenden"
echo ""

wait

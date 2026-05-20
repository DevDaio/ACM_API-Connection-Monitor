# Übung: Mini-Projekt – Eigene Erweiterung

**Level:** 4 – Expert

## Aufgabe
Entwickle eine eigene Erweiterung für den ACM API Connection Monitor. Wähle **eine** der folgenden Ideen:

## Idee A: Benachrichtigungen
- Wenn ein Endpoint down geht → Email oder Webhook senden
- Neue DB-Tabelle: `alert (endpointid, webhook_url, email)`
- Monitoring-Loop löst Alert aus bei Status-Change

## Idee B: Dashboard mit Charts
- Ersetze die Sparkline durch echte Charts (z.B. Chart.js oder Recharts)
- Zeige: Uptime letzte 24h/7d/30d
- Response-Zeiten als Liniendiagramm

## Idee C: Multi-User + Teams
- Teams von Usern (mehrere User teilen sich Endpoints)
- Neue Tabellen: `team`, `teamuser`, `teamendpoint`
- Berechtigungen: Owner / Editor / Viewer

## Idee D: Eigenes
- Du hast eine bessere Idee? Nur zu!

## Abgabe
- Vollständiger Code (Backend + Frontend)
- Tests
- Kurze Erklärung der Architektur-Entscheidungen

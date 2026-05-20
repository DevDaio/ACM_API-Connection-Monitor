# Glossar

| Begriff | Definition |
|---------|------------|
| **Axum** | Rust-Webframework mit async-Handlern, State-Management und Middleware |
| **CORS** | Cross-Origin Resource Sharing – erlaubt Browser-Anfragen zwischen verschiedenen Origins |
| **Endpoint** | Eine URL/API-Route, die vom Monitor überwacht wird |
| **Handler** | Async-Funktion in Axum, die eingehende HTTP-Requests verarbeitet |
| **Hook (React)** | useState, useEffect, useRef – Funktionen zum Verwalten von State/Lebenszyklus |
| **Intervall** | Zeitabstand in Sekunden, in dem ein Endpoint geprüft wird |
| **JWT** | JSON Web Token – (optional) Authentifizierungsstandard, hier nicht verwendet |
| **LED** | Status-Licht (grün = Running, rot = Down, grau = Unknown) |
| **M:N** | Many-to-Many-Datenbankbeziehung (User ↔ Endpoint) |
| **Monitoring** | Automatische, regelmäßige Prüfung von Endpoints auf Erreichbarkeit |
| **Pool (PgPool)** | PostgreSQL-Verbindungspool – wiederverwendbare DB-Connections |
| **reqwest** | Rust-HTTP-Client für ausgehende Requests |
| **serde** | Rust-Serialisierungsbibliothek (JSON ↔ Struct) |
| **sqlx** | Rust-SQL-Bibliothek mit asynchronem PostgreSQL-Support |
| **State (Axum)** | Geteilter Anwendungszustand, der an alle Handler weitergegeben wird |
| **Theme** | Farbschema (Lava Red, Hacker Green, Void Purple) via CSS-Variablen |
| **Tokio** | Async-Runtime für Rust – ermöglicht nebenläufige Tasks |
| **Uptime** | Zeit seit dem letzten Statuswechsel (Running/Unknown → Down oder umgekehrt) |

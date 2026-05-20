# Übung: JWT-Authentifizierung einbauen

**Level:** 4 – Expert

## Aufgabe
Ersetze das einfache Email/Passwort-Login durch JWT-basierte Authentifizierung.

## Anforderungen
1. Login gibt JWT-Token zurück (statt userid)
2. Jeder Request muss das Token im `Authorization: Bearer <token>`-Header mitsenden
3. Middleware prüft Token vor jedem geschützten Endpoint
4. Token enthält: userid, email, exp (Ablaufdatum)

## Hinweise
- Crate: `jsonwebtoken` (JWT) + `once_cell` (lazy statics)
- Secret aus `.env` lesen (`JWT_SECRET`)
- axum-Middleware für Token-Validierung

## Struktur
```
/login → POST → { "token": "eyJ..." }
/home?token=... → GET → { endpoints }
```

## Erwartete Änderungen
1. **main.rs**: JWT-Secret laden, Middleware registrieren
2. **Neue Datei**: `auth.rs` mit Token-Erstellung + Validation
3. **api.js**: Token speichern + mitsenden
4. **App.jsx**: Token-basierte Authentifizierung

## Bonus
Implementiere Refresh-Tokens.

# ACM API Connection Monitor — Große Leinwand

> Vollständige Codebasis-Visualisierung. 13 Routen · 5 DB-Tabellen · React+Axum+Rust · Echtzeit-Monitoring.

---

## 1. Gesamtbild — Architektur-Übersicht

```mermaid
graph TD
    U["Browser / Nutzer"] -->|"HTTP :5173"| FE["Frontend (React + Vite)"]
    FE -->|"HTTP/JSON :3000"| GW["Axum-Gateway (main.rs)"]
    GW -->|"sqlx (PgPool)"| DB[("PostgreSQL-Datenbank")]
    GW -->|"RwLock"| SESS["Sitzungen (HashMap &lt;Token, Nutzer-ID&gt;)"]
    GW -->|"tokio::spawn"| ML["Überwachungs-Schleife (async_services.rs)"]
    ML -->|"HTTP / TCP / ICMP (je nach check_type)"| TGT["Ziel-APIs / Hosts / Ports"]
    ML -->|"INSERT INTO log"| DB
    GW -->|".layer(cors)"| CORS["CORS: jede Quelle, jeder Header"]

    style FE fill:#1a3a5c,color:#fff
    style GW fill:#5c2d91,color:#fff
    style DB fill:#2d5a27,color:#fff
    style ML fill:#8b4513,color:#fff
    style SESS fill:#a04040,color:#fff
```

**Komponenten:**

| Ebene | Technologie | Datei(en) |
|-------|-------------|-----------|
| Frontend | React 19 + Vite + Tailwind | `Frontend/src/` |
| Backend | Axum (Rust, asynchron) | `Backend/src/main.rs` |
| Handler | Axum-Routen-Handler | `Backend/src/handlers.rs` |
| DB-Ebene | sqlx (zur Compilezeit geprüft) | `Backend/src/service_modules/async_services.rs` |
| Überwachung | Tokio-Task + reqwest / TcpStream / system ping | `async_services.rs:run_monitoring_loop()` |
| Datenbank | PostgreSQL | `DB/createTables.sql` / `main.rs:46-54` |

---

## 2. Vollständige Routenkarte — Alle 13 Routen mit Authentifizierungstoren

```mermaid
flowchart LR
    subgraph PUBLIC["Öffentlich (kein Token nötig) — 3 Routen"]
        HC("GET /acm"):::pub -->|"gibt {status:ok} zurück"| HCO["{status: ok}"]
        LOGIN("POST /acm/login"):::pub -->|"E-Mail + Passwort → bcrypt prüfen"| LOGINO["200: {Nutzer-ID, E-Mail, Token}"]
        CA("POST /acm/createAccount"):::pub -->|"E-Mail + Passwort → bcrypt hashen"| CAO["200: {Nutzer-ID, E-Mail, Token}"]
    end

    subgraph GESCHUETZT["Geschützt (Bearer-Token nötig) — 6 Routen"]
        H("GET /acm/home"):::ges -->|"Token → Nutzer-ID → JOIN-Abfrage"| HO["Liste &lt;EndpointExtended&gt;"]
        U("GET /acm/user"):::ges -->|"Token → Nutzer-ID"| UO["Nutzer"]
        CP("PUT /acm/user/changePassword"):::ges -->|"Token + altes PW + neues PW"| CPO["{status: ok}"]
        CE("PUT /acm/user/changeEmail"):::ges -->|"Token + neue E-Mail"| CEO["{status: ok}"]
        DA("DELETE /acm/user/deleteAccount"):::ges -->|"Token"| DAO["{status: ok}"]
        AE("PUT /acm/addEndpoint"):::ges -->|"Token + URL"| AEO["{endpointid}"]
    end

    subgraph UNGESCHUETZT["Ungeschützt (kein Token, Body-Parameter) — 4 Routen"]
        SI("PUT /acm/setIntervall"):::ung -->|"Endpunkt-ID + Sekunden (UPSERT)"| SIO["{status: ok}"]
        DC("PUT /acm/deleteConfirm"):::ung -->|"Endpunkt-ID → Kaskaden-Löschung"| DCO["{status: ok}"]
        UE("PUT /acm/updateEndpoint"):::ung -->|"Endpunkt-ID + URL + Log-Eintrag"| UEO["{status: ok}"]
        LG("GET /acm/log"):::ung -->|"?id=N → SELECT aus log"| LGO["Liste &lt;Log&gt;"]
    end

    classDef pub fill:#1a5c1a,color:#fff
    classDef ges fill:#5c2d91,color:#fff
    classDef ung fill:#8b4513,color:#fff
```

**Authentifizierungs-Mechanismus:** Jeder geschützte Handler ruft `get_userid_from_token()` auf (handlers.rs:23). Liest `Authorization: Bearer <token>` → Nachschlagen in `AppState.sessions` (RwLock). Gibt **401** bei fehlendem/ungültigem Token.

---

## 3. Anfrage-Lebenszyklus — Login → DB → Token → Dashboard

```mermaid
sequenceDiagram
    participant N as Nutzer
    participant LP as LandingPage.jsx
    participant API as api.js
    participant FEZ as useAppState.js
    participant BE as Axum-Handler (handlers.rs)
    participant AS as async_services.rs
    participant DB as PostgreSQL
    participant SESS as Sitzungen-HashMap

    N->>LP: E-Mail + Passwort eingeben
    LP->>API: onLogin(E-Mail, Passwort)
    API->>BE: POST /acm/login {E-Mail, Passwort}
    BE->>AS: get_user_by_email(&pool, E-Mail)
    AS->>DB: SELECT * FROM "user" WHERE emailadress = $1
    DB-->>AS: User {Nutzer-ID, E-Mail, Passwort-Hash}
    AS-->>BE: Ok(Some(Nutzer))

    BE->>BE: bcrypt::verify(Passwort, Hash)
    alt Ungültiges Passwort
        BE-->>API: 401 "Ungültige E-Mail oder Passwort"
        API-->>LP: Fehler werfen
        LP-->>N: Fehlermeldung anzeigen
    end

    BE->>BE: uuid::Uuid::new_v4() → Token
    BE->>SESS: sessions.write().insert(Token, Nutzer-ID)
    BE-->>API: 200 {Nutzer-ID, E-Mail-Adresse, Token}
    API->>API: setToken(Token) → localStorage.setItem('acm_token', Token)
    API-->>LP: Daten zurückgeben
    LP->>FEZ: handleLogin wird aufgelöst
    FEZ->>FEZ: setUser({Nutzer-ID, E-Mail})
    FEZ->>FEZ: useEffect → api.getHome()
    API->>BE: GET /acm/home (Authorization: Bearer <Token>)
    BE->>SESS: get_userid_from_token(Header, Zustand)
    SESS-->>BE: Nutzer-ID
    BE->>AS: get_user_endpoints(&pool, Nutzer-ID)
    AS->>DB: Komplexer LEFT JOIN + LATERAL-Abfrage
    DB-->>AS: EndpointExtended (JOIN-Ergebnis)
    AS-->>BE: Endpunkte (gefiltert)
    BE-->>API: 200 [EndpunktErweitert]
    API-->>FEZ: setEndpoints(mapEndpoints(Daten))
    Note over FEZ: Rendert Dashboard mit EndpointCards
```

**Schlüssel-Details:**
- Token ist ein **UUIDv4**, wird bei jedem Login neu erzeugt
- Frontend speichert Token im **localStorage** (`acm_token`) — überlebt Seiten-Neuladung
- Bei **401** (ungültiger/abgelaufener Token): `setToken(null)` + `setUser(null)` → zurück zu LandingPage
- Passwort-Hashing: **bcrypt** mit DEFAULT_COST (~10-12 Runden)

---

## 4. React-Komponentenbaum — Hierarchie

```mermaid
graph TD
    APP["App.jsx"] --> TPC["ThemeContext.jsx<br/>&lt;ThemeProvider&gt;"]
    TPC -->|"!Nutzer"| LP["LandingPage.jsx<br/>Login-Formular (Terminal-Design)"]
    TPC -->|"!Nutzer"| CAM["CreateAccountModal.jsx<br/>E-Mail + Passwort + Bestätigung"]

    TPC -->|"Nutzer"| DASH["Dashboard.jsx<br/>Endpunkt-Tabelle + Kopfzeile"]
    DASH --> TS["ThemeSwitcher.jsx<br/>LAVA / HACKER-GRÜN / LEER-LILA"]
    DASH --> EC["EndpointCard.jsx × N<br/>Eine Zeile pro Endpunkt"]
    EC --> SP["Sparkline.jsx<br/>Letzte 30 Status (SVG-Linie)"]

    TPC -->|"Nutzer"| AEM["AddEndpointModal.jsx<br/>URL + HH:MM:SS Intervall"]
    TPC -->|"Nutzer"| SIM["SetIntervallModal.jsx<br/>HH:MM:SS für bestehenden EP"]
    TPC -->|"Nutzer"| DCM["DeleteConfirmModal.jsx<br/>Bist du sicher? Ablehnen / Bestätigen"]
    TPC -->|"Nutzer"| ASM["AccountSettingsModal.jsx<br/>PW ändern / E-Mail ändern / Account löschen"]
    TPC -->|"Nutzer"| LM["LogModal.jsx<br/>Log-Tabelle + Filter (alle/oben/unten + Datum)"]
    TPC -->|"Nutzer"| EUM["EditUrlModal.jsx<br/>URL bearbeiten + speichern"]

    MODAL_BASE["Modal.jsx (Basis)<br/>Overlay + Titel-Leiste + Inhalt"] --> CAM
    MODAL_BASE --> AEM
    MODAL_BASE --> SIM
    MODAL_BASE --> DCM
    MODAL_BASE --> ASM
    MODAL_BASE --> LM
    MODAL_BASE --> EUM

    HOOK["useAppState.js<br/>Zentraler Zustand + API-Logik"] -.-> APP
    HOOK -.->|"Callback-Props"| DASH
    HOOK -.->|"Callback-Props"| AEM
    HOOK -.->|"Callback-Props"| SIM
    HOOK -.->|"Callback-Props"| DCM
    HOOK -.->|"Callback-Props"| ASM
    HOOK -.->|"Callback-Props"| LM
    HOOK -.->|"Callback-Props"| EUM

    HELF["utils/helpers.js<br/>fmtDuration, fmtInterval, normalizeUrl, mapEndpoints"] -.-> DASH
    HELF -.-> EC
    HELF -.-> SP

    API["api.js<br/>HTTP-Client + Token-Verwaltung"] -.-> HOOK

    style APP fill:#1e3a5f,color:#fff
    style HOOK fill:#5c2d91,color:#fff
    style MODAL_BASE fill:#2d5a27,color:#fff
```

**Zustands-Verteilung:**

| Zustand | Besitzer | Prop-Drilling-Tiefe |
|---------|---------|---------------------|
| Nutzer, Endpunkte, Hauptschalter | `useAppState`-Hook (in App.jsx) | 1-2 Ebenen |
| Modal-sichtbar (showX) | `useAppState`-Hook | An Modals übergeben |
| Theme | `ThemeContext.jsx` (Context-API) | Global via `useTheme()` |
| Endpunkt-Logs | `useAppState` (logEntries) | An LogModal |
| _Token (lokal) | api.js (Modul-Gültigkeitsbereich) | Kein React-Zustand |

---

## 5. Datenfluss: Endpunkt Hinzufügen — Vollständiger PUT-Durchlauf

```mermaid
sequenceDiagram
    participant N as Nutzer
    participant AEM as AddEndpointModal.jsx
    participant HOOK as useAppState.js
    participant API as api.js
    participant BE as Axum-Handler
    participant AS as async_services.rs
    participant DB as PostgreSQL

    N->>AEM: Klick "+ ENDPUNKT_HINZUFÜGEN" → Modal öffnet
    N->>AEM: Wählt Check-Typ (HTTP / TCP / ICMP)
    N->>AEM: Füllt URL + Intervall (HH:MM:SS)
    AEM->>AEM: h*3600 + m*60 + s = Gesamt-Sekunden
    AEM->>HOOK: onSubmit(roheURL, gesamtSekunden, checkType)

    HOOK->>HOOK: checkType === 'http' ? normalizeUrl(roheURL) : roheURL.trim()
    Note over HOOK: HTTP: Protokoll ergänzen<br/>TCP/ICMP: roh lassen (host:port)

    HOOK->>API: api.addEndpoint(URL, checkType)
    API->>API: request("PUT", "/addEndpoint", {url, check_type})
    Note over API: setzt Authorization: Bearer <_Token>
    API->>BE: PUT /acm/addEndpoint<br/>{Authorization, "url":"..."}

    BE->>BE: get_userid_from_token(Header, Zustand)
    Note over BE: Extrahiert Token aus Header → Sitzungen HashMap → Nutzer-ID

    BE->>AS: add_endpoint(&pool, Nutzer-ID, &URL)
    AS->>DB: SELECT COUNT(*) FROM endpoint
    DB-->>AS: Anzahl
    alt Anzahl == 0
        AS->>DB: ALTER TABLE endpoint ALTER COLUMN endpointid RESTART WITH 1
    end
    AS->>DB: INSERT INTO endpoint (url, check_type) VALUES ($1, $2) RETURNING endpointid
    DB-->>AS: Endpunkt-ID
    AS->>DB: INSERT INTO userendpoint (Nutzer-ID, Endpunkt-ID) VALUES ($1, $2)
    DB-->>AS: OK
    AS-->>BE: Endpunkt-ID

    BE-->>API: 200 {endpointid: N}

    HOOK->>HOOK: api.addEndpoint wird aufgelöst → Daten.endpointid
    HOOK->>API: api.setIntervall(Daten.endpointid, Gesamt-Sekunden)
    API->>BE: PUT /acm/setIntervall {endpointid, Sekunden}
    BE->>AS: set_intervall(&pool, Endpunkt-ID, Sekunden)
    AS->>DB: INSERT INTO intervall ... ON CONFLICT DO UPDATE
    DB-->>AS: OK
    BE-->>API: 200 {status: ok}

    HOOK->>HOOK: refreshEndpoints() → api.getHome()
    HOOK->>API: GET /acm/home (Bearer)
    API->>BE: GET /acm/home
    BE->>DB: Komplexer JOIN: endpoint + userendpoint + intervall + log (LATERAL)
    DB-->>BE: EndpointExtended[] mit Status, Sparkline, Intervall
    BE-->>API: 200 [...]
    API-->>HOOK: setEndpoints(mapEndpoints(Daten))

    HOOK->>HOOK: pollUntilReady() → 8× alle 2s refreshEndpoints

    AEM->>AEM: Felder leeren + onClose()
    Note over N: Neuer Endpunkt in Tabelle sichtbar<br/>Überwachung beginnt beim nächsten Schleifen-Durchlauf
```

**Wichtige Details:**
- `normalizeUrl()` erkennt: `http://`, `https://`, `localhost`, IPv4, IPv6, und fügt `https://` als Standard hinzu
- Beim ersten Endpunkt wird die Sequenz zurückgesetzt (`RESTART WITH 1`)
- `setIntervall` verwendet **UPSERT** (`ON CONFLICT DO UPDATE`)
- Nach dem Hinzufügen: **Abfragen** mit 8 Versuchen alle 2 Sekunden (max. 16s) bis Daten aktuell

---

## 6. Überwachungs-Schleife — Timer-basierte Endpunkt-Prüfung

```mermaid
flowchart TD
    START(["tokio::spawn (Hintergrund-Task)"]) --> BUILD["reqwest::Client::builder()<br/>.timeout(10s)<br/>.danger_accept_invalid_certs(true)"]
    BUILD --> SCHLEIFE

    subgraph SCHLEIFE ["Hauptschleife (alle 5 Sekunden)"]
        ABFR["SELECT i.endpointid, i.seconds, e.url, e.check_type<br/>FROM intervall i JOIN endpoint e<br/>USING (endpointid)"]
        ABFR --> FEHLER{"DB-Fehler?"}
        FEHLER -->|"Ja"| FEHLER_LOG["eprintln + 10s schlafen"]
        FEHLER_LOG --> SCHLAF

        FEHLER -->|"Nein"| FUER_JEDEN["Für jeden Endpunkt"]

        FUER_JEDEN --> LETZTER_CHECK{"last_checked.get(&ep.endpointid)?"}
        LETZTER_CHECK -->|"None (noch nie)"| PING["🟟 Sofort prüfen"]
        LETZTER_CHECK -->|"Some(Letzter)"| ABGELAUFEN{"last.elapsed() >=<br/>ep.seconds?"}
        ABGELAUFEN -->|"Nein (noch nicht fällig)"| UEBERSPRING["⏭ Überspringen"]
        UEBERSPRING --> NAECHSTER["Nächster Endpunkt"]

        ABGELAUFEN -->|"Ja"| PING

        PING --> CHECK_TYPE{"check_type?"}
        CHECK_TYPE -->|"http"| HTTP_GET["reqwest::get(&ep.url)"]
        CHECK_TYPE -->|"tcp"| TCP["TcpStream::connect(addr)"]
        CHECK_TYPE -->|"icmp"| ICMP["system ping -c1 -W3 host"]
        HTTP_GET --> HTTP_OK{"2xx?"}
        HTTP_OK -->|"Ja"| UP
        HTTP_OK -->|"Nein/Err"| DOWN
        TCP --> TCP_OK{"Connected?"}
        TCP_OK -->|"Ja"| UP
        TCP_OK -->|"Nein"| DOWN
        ICMP --> ICMP_OK{"Exit 0?"}
        ICMP_OK -->|"Ja"| UP
        ICMP_OK -->|"Nein"| DOWN

        UP --> EINFUEG["INSERT INTO log (endpointid, status, url)<br/>VALUES ($1, $2, $3)"]
        DOWN --> EINFUEG

        EINFUEG --> EINFUEG_FEHLER{"Einfügen-Fehler?"}
        EINFUEG_FEHLER -->|"Ja"| LOG_FEHLER["eprintln Log-Einfügen-Fehler"]
        EINFUEG_FEHLER -->|"Nein"| AKTUALISIEREN["last_checked.insert(ep.endpointid, Instant::now())"]
        LOG_FEHLER --> AKTUALISIEREN
        AKTUALISIEREN --> NAECHSTER
        NAECHSTER --> ALLE_DURCH{"Alle durch?"}
        ALLE_DURCH -->|"Nein"| FUER_JEDEN
        ALLE_DURCH -->|"Ja"| SCHLAF["tokio::time::sleep(5s)"]
    end

    SCHLAF --> ABFR

    style START fill:#5c2d91,color:#fff
    style SCHLEIFE fill:#1e3a5f,color:#fff
    style PING fill:#8b4513,color:#fff
    style UP fill:#1a5c1a,color:#fff
    style DOWN fill:#8b0000,color:#fff
```

**Überwachungs-Details:**

| Aspekt | Wert |
|--------|------|
| Schleifen-Takt | 5 Sekunden (fest) |
| Individuelles Intervall | Pro Endpunkt konfigurierbar in `intervall.seconds` |
| Check-Typen | `http` (HTTP-GET), `tcp` (TCP-Verbindung), `icmp` (ICMP-Ping) |
| HTTP-Zeitüberschreitung | 10 Sekunden |
| TCP-Zeitüberschreitung | 5 Sekunden |
| SSL | Selbst-signierte Zertifikate erlaubt (`danger_accept_invalid_certs(true)`) |
| Log-Format | `endpointid, status (bool/NULL), url, statusdate DATE, statustime TIME` |
| Zustands-Verfolgung | `HashMap<i32, Instant>` im Speicher (keine DB) |
| Fehler-Behandlung | DB-Fehler → 10s Pause, Log-Fehler → eprintln + continue |

---

## 7. Datenbank-Schema — 5 Tabellen mit Beziehungen

```mermaid
classDiagram
    class Benutzer {
        +INTEGER userid PK
        +VARCHAR(100) emailadress UNIQUE
        +VARCHAR(100) password
    }

    class Endpunkt {
        +INTEGER endpointid PK
        +VARCHAR(300) url
        +VARCHAR(10) check_type
    }

    class BenutzerEndpunkt {
        +INTEGER userid PK, FK
        +INTEGER endpointid PK, FK
    }

    class Intervall {
        +INTEGER endpointid PK, FK
        +INTEGER seconds
    }

    class Log {
        +INTEGER endpointid FK
        +BOOLEAN status [nullable]
        +DATE statusdate DEFAULT CURRENT_DATE
        +TIME statustime DEFAULT CURRENT_TIME
        +VARCHAR(300) url [nullable]
    }

    Benutzer "1" --> "*" BenutzerEndpunkt : "userid → userid ON DELETE CASCADE"
    Endpunkt "1" --> "*" BenutzerEndpunkt : "endpointid → endpointid ON DELETE CASCADE"
    Endpunkt "1" --> "0..1" Intervall : "endpointid → endpointid ON DELETE CASCADE"
    Endpunkt "1" --> "*" Log : "endpointid → endpointid ON DELETE CASCADE"
```

**SQL-CREATE-Tabellen (aus main.rs / createTables.sql):**

```sql
-- 1. Benutzer (Anführungszeichen wegen SQL-Reserved-Wort)
CREATE TABLE IF NOT EXISTS "user" (
    userid      INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    emailadress VARCHAR(100) NOT NULL UNIQUE,
    password    VARCHAR(100) NOT NULL
);

-- 2. Endpunkt (check_type: http | tcp | icmp)
CREATE TABLE IF NOT EXISTS endpoint (
    endpointid  INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    url         VARCHAR(300) NOT NULL,
    check_type  VARCHAR(10) NOT NULL DEFAULT 'http'
);

-- 3. Benutzer-Endpunkt (n:m Junction-Tabelle)
-- CASCADE DELETE: Löschung eines Benutzers/Endpunkts löscht auch dessen Verknüpfungen
CREATE TABLE IF NOT EXISTS userendpoint (
    userid      INTEGER NOT NULL,
    endpointid  INTEGER NOT NULL,
    PRIMARY KEY (userid, endpointid),
    FOREIGN KEY (userid)    REFERENCES "user"(userid)    ON DELETE CASCADE,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

-- 4. Intervall (1:1 mit Endpunkt, UPSERT via ON CONFLICT)
CREATE TABLE IF NOT EXISTS intervall (
    endpointid  INTEGER PRIMARY KEY,
    seconds     INTEGER NOT NULL,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

-- 5. Log (1:n mit Endpunkt, nullable status = URL-Edit-Event)
CREATE TABLE IF NOT EXISTS log (
    endpointid  INTEGER NOT NULL,
    status      BOOLEAN,                    -- true=up, false=down, NULL=URL-edit
    url         VARCHAR(300),               -- URL zum Zeitpunkt des Eintrags
    statusdate  DATE NOT NULL DEFAULT CURRENT_DATE,
    statustime  TIME NOT NULL DEFAULT CURRENT_TIME,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

-- Migrationen (bei jedem Start ausgeführt):
ALTER TABLE log ADD COLUMN IF NOT EXISTS url VARCHAR(300);
ALTER TABLE log ALTER COLUMN status DROP NOT NULL;
ALTER TABLE endpoint ADD COLUMN IF NOT EXISTS check_type VARCHAR(10) NOT NULL DEFAULT 'http';
```

**Komplexe Dashboard-Abfrage** (in `get_user_endpoints`, async_services.rs:80-114):

```sql
SELECT e.endpointid, e.url, e.check_type,
       l.status, l.statusdate, l.statustime,
       -- Dauer seit letztem Statuswechsel in Sekunden
       CASE WHEN l.statusdate IS NOT NULL THEN
         EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - (
           SELECT COALESCE(
             MAX(l2.statusdate + l2.statustime),
             (SELECT MIN(l3.statusdate + l3.statustime) FROM log l3 WHERE l3.endpointid = e.endpointid)
           ) FROM log l2 WHERE l2.endpointid = e.endpointid AND l2.status != l.status
         )))::integer
       ELSE NULL END AS duration_seconds,
       i.seconds AS interval_seconds,
       -- Letzte 30 Status-Werte als Array (für Sparkline)
       COALESCE((SELECT ARRAY(SELECT status FROM (
           SELECT status, statusdate, statustime FROM log
           WHERE endpointid = e.endpointid AND status IS NOT NULL
           ORDER BY statusdate DESC, statustime DESC LIMIT 30
       ) sub ORDER BY statusdate ASC, statustime ASC)), ARRAY[]::BOOLEAN[]) AS status_history
FROM endpoint e
JOIN userendpoint ue ON e.endpointid = ue.endpointid
LEFT JOIN intervall i ON i.endpointid = e.endpointid
LEFT JOIN LATERAL (
    SELECT status, statusdate, statustime
    FROM log WHERE endpointid = e.endpointid
    ORDER BY statusdate DESC, statustime DESC LIMIT 1
) l ON true
WHERE ue.userid = $1;
```

---

## 8. Sitzungs-Zustand — In-Memory-Token-Lebensdauer

```mermaid
flowchart TD
    LOGIN["POST /acm/login<br/>oder /createAccount"] --> GEN_TOKEN["uuid::Uuid::new_v4()<br/>→ String-Token"]
    GEN_TOKEN --> SPEICH["sessions.write().insert(Token, Nutzer-ID)<br/>(HashMap&lt;String, i32&gt;)"]
    SPEICH --> ANTW["Antwort: {Nutzer-ID, E-Mail, Token}"]
    ANTW --> CLIENT_SPEICH["localStorage.setItem('acm_token', Token)<br/>api.js: _Token = t"]

    subgraph CLIENT_SEITE["Frontend (api.js)"]
        ANFR["Jeder API-Request"] --> PRUEF_TOKEN{"_Token gesetzt?"}
        PRUEF_TOKEN -->|"Ja"| HEADER_HINZ["opts.headers.Authorization =<br/>`Bearer ${_Token}`"]
        PRUEF_TOKEN -->|"Nein"| KEIN_HEADER["Kein Auth-Header"]
        HEADER_HINZ --> SENDEN["fetch(BASE + Pfad, opts)"]
        SENDEN --> PRUEF_401{"Antwort-Status 401?"}
        PRUEF_401 -->|"Ja"| TOKEN_LOESCH["setToken(null)<br/>localStorage.removeItem('acm_token')"]
        PRUEF_401 -->|"Nein"| NORMAL["Normal verarbeiten"]
        TOKEN_LOESCH --> AUTH_FEHLER["useAppState handleAuthError()<br/>→ setUser(null), setEndpoints([])"]
    end

    subgraph SERVER_SEITE["Backend (handlers.rs)"]
        GESCHUETZT_ANFR["Geschützter Handler"] --> PARSE_HEADER["headers.get('authorization')<br/>→ .strip_prefix('Bearer ')"]
        PARSE_HEADER -->|"Fehlt/ungültig"| 401["401 Unberechtigt<br/>(Fehlender oder ungültiger Auth-Header)"]
        PARSE_HEADER -->|"Token extrahiert"| MAP_NACHSCHLAG["state.sessions.read().get(Token)"]
        MAP_NACHSCHLAG -->|"Nicht gefunden"| 401_ABGEL["401 Unberechtigt<br/>(Ungültiger oder abgelaufener Session-Token)"]
        MAP_NACHSCHLAG -->|"Nutzer-ID gefunden"| FORTS["Handler ausführen mit Nutzer-ID"]
    end

    subgraph ABMELDEN["Abmelden / Token-Ende"]
        MANUELL["Nutzer klickt 'Exit'"] --> HOOK_ABMELD["handleLogout()"]
        HOOK_ABMELD --> API_ABMELD["api.setToken(null)"]
        API_ABMELD --> LOCAL_LOESCH["localStorage.removeItem('acm_token')"]
        LOCAL_LOESCH --> ZUSTAND_LOESCH["setUser(null)<br/>setEndpoints([])"]

        SEITEN_NEULAD["Seiten-Neuladung"] --> TOKEN_WIEDERHER["const saved = localStorage.getItem('acm_token')<br/>_Token = saved"]
        TOKEN_WIEDERHER -->|"Token vorhanden"| ANFR

        UNGENUTZ["Token existiert Server-seitig<br/>bis zum Neustart weiter"]
    end

    AUTH_FEHLER -.->|"401 von jedem Request"| HOOK_ABMELD

    style CLIENT_SEITE fill:#1e3a5f,color:#fff
    style SERVER_SEITE fill:#5c2d91,color:#fff
    style ABMELDEN fill:#8b0000,color:#fff
```

**Session-Eigenschaften:**

| Aspekt | Verhalten |
|--------|-----------|
| Token-Format | UUID v4 (String, z.B. `"550e8400-e29b-41d4-a716-446655440000"`) |
| Speicherort Server | `Arc<RwLock<HashMap<String, i32>>>` in `AppState` |
| Speicherort Client | `localStorage`-Schlüssel `acm_token` + Modul-Variable `_token` |
| Lebensdauer Server | **Permanent bis Server-Neustart** (kein TTL/Verfall implementiert) |
| Lebensdauer Client | Permanent in localStorage (überlebt Tabs + Neuladungen) |
| Ungültigmachung | Nie (Server löscht nie aus HashMap) |
| Erneuerung | Jeder Login erzeugt neuen Token (alter bleibt gültig!) |
| Abmelden | Entfernt nur client-seitiges localStorage — Server-Token bleibt |

---

## 9. Frontend-Abfrage und Echtzeit-Mechanismus

```mermaid
flowchart LR
    subgraph useEffect_START["useEffect (Start / Nutzer-Wechsel)"]
        M1["api.getHome()"] --> M2["setEndpoints(mapEndpoints(Daten))"]
    end

    subgraph useEffect_SEKUNDE["useEffect: Sekunden-Tick"]
        T1["setInterval 1000ms"] --> T2["vorher.map(ep =><br/>dauerSekunden + 1)"]
    end

    subgraph useEffect_ABFRAGE["useEffect: Dashboard-Abfrage"]
        P1["setInterval 10000ms"] --> P2{"anyModalRef.current ? (Modal offen)"}
        P2 -->|"Ja"| ABFRAGE_UEBERSP["Überspringen (keine Aktualisierung)"]
        P2 -->|"Nein"| P3["api.getHome()"]
        P3 --> P4["setEndpoints(mapEndpoints(Daten))"]
    end

    subgraph bereitBis["bereitBis() (nach Änderung)"]
        PU1["clearInterval alt"] --> PU2["setInterval 2000ms × 8 Versuche"]
        PU2 --> PU3["refreshEndpoints()"]
        PU3 --> PU4["Versuche ≥ 8? → Stopp"]
    end

    subgraph LogModal_ABFRAGE["LogModal: Live-Logs"]
        L1["useEffect: setInterval 5000ms"] --> L2["fetchLog(ausgewählter.endpointid)"]
        L2 --> L3["setLogEntries(Einträge)"]
    end

    useEffect_START --> useEffect_SEKUNDE
    useEffect_SEKUNDE --> useEffect_ABFRAGE
    useEffect_ABFRAGE --> bereitBis
    bereitBis --> LogModal_ABFRAGE
```

**Abfrage-Strategien:**

| Typ | Intervall | Auslöser | Stopp |
|-----|-----------|---------|-------|
| Dashboard | 10s | `useEffect` nach Login | Modal offen → aussetzen |
| Zeit-Korrektur | 1s | `useEffect` | Nutzer-Abmeldung |
| Nach-Änderung | 2s (max 16s) | `bereitBis()` nach Hinzufügen/Bearbeiten/Löschen | 8 Versuche erreicht |
| Log-Live | 5s | LogModal offen | LogModal schließt |

---

## 10. Start / Konfiguration / Initialisierungs-Sequenz

```mermaid
flowchart TD
    START["main.rs: #[tokio::main]"] --> DOTENV["dotenv::dotenv().ok()<br/>Lädt .env-Datei"]
    DOTENV --> DATABASE_URL["env::var('DATABASE_URL')<br/>→ Rückfall: postgres://admin:admin@localhost:5432/mydb"]
    DATABASE_URL --> POOL["PgPoolOptions::new()<br/>.max_connections(5)<br/>.connect(&database_url)"]
    POOL --> TABELLEN_ANLEG["CREATE TABLE IF NOT EXISTS × 5<br/>+ ALTER TABLE Migrationen"]
    TABELLEN_ANLEG --> UEBERWACH_START["tokio::spawn(async move {<br/>run_monitoring_loop(pool).await<br/>})"]
    UEBERWACH_START --> ZUSTAND_BAU["Arc::new(AppState {<br/>pool,<br/>sessions: RwLock::new(HashMap::new())<br/>})"]
    ZUSTAND_BAU --> CORS["CorsLayer::new()<br/>.allow_origin(Any)<br/>.allow_methods(GET,POST,PUT,DELETE)<br/>.allow_headers(Any)"]
    CORS --> ROUTEN["Router::new()<br/>.route('/acm', ...) × 13<br/>.layer(cors)<br/>.with_state(zustand)"]
    ROUTEN --> BINDEN["BACKEND_HOST / BACKEND_PORT<br/>→ Standard: 0.0.0.0:3000"]
    BINDEN --> BEDIEN["axum::serve(listener, app).await"]

    DOTENV -.->|".env-Variablen"| HOST["BACKEND_HOST<br/>BACKEND_PORT<br/>DATABASE_URL"]

    style START fill:#5c2d91,color:#fff
    style BEDIEN fill:#1a5c1a,color:#fff
    style CORS fill:#8b4513,color:#fff
```

**Konfig-Übersicht:**

| Variable | Standard | Zweck |
|----------|---------|-------|
| `DATABASE_URL` | `postgres://admin:admin@localhost:5432/mydb` | PostgreSQL-Verbindung |
| `BACKEND_HOST` | `0.0.0.0` | Server-Bind-IP |
| `BACKEND_PORT` | `3000` | Server-Port |
| `VITE_API_URL` | `/acm` (Vite-Proxy) | Frontend-API-Basis-URL |

**CORS-Konfiguration:**
- `allow_origin(Any)` — jede Domain darf anfragen (Entwicklung)
- `allow_methods([GET, POST, PUT, DELETE])` — alle CRUD-Operationen
- `allow_headers(Any)` — beliebige Header (v.a. `Authorization` für Bearer-Token)

---

## Anhang: Dateistruktur

```
ACM_API-Connection-Monitor/
├── Backend/
│   └── src/
│       ├── main.rs                     # Server-Start, CORS, Routen, DB-Init, Überwachungs-Spawn
│       ├── handlers.rs                 # 13 Axum-Handler (öffentlich + geschützt)
│       ├── types.rs                    # AppState, Request/Response-Structs
│       └── service_modules/
│           ├── mod.rs                  # Modul-Deklaration
│           └── async_services.rs       # DB-Operationen, Überwachungs-Schleife, CRUD, JOIN-Query
├── Frontend/
│   └── src/
│       ├── App.jsx                     # Wurzel-Komponente, Bedingtes Rendering
│       ├── App.css                     # Basis-Styles
│       ├── api.js                      # HTTP-Client, Token-Verwaltung, 11 API-Funktionen
│       ├── ThemeContext.jsx             # Theme-Provider (lava/green/purple)
│       ├── hooks/
│       │   └── useAppState.js           # Zentraler Zustand + alle Callback-Handler
│       ├── components/
│       │   ├── Modal.jsx               # Basis-Overlay-Komponente
│       │   ├── LandingPage.jsx          # Login-Bildschirm (Terminal-Design)
│       │   ├── CreateAccountModal.jsx   # Registrierungs-Formular
│       │   ├── Dashboard.jsx            # Hauptansicht mit Tabelle
│       │   ├── EndpointCard.jsx         # Endpunkt-Tabellenzeile
│       │   ├── Sparkline.jsx            # SVG-Mini-Chart (letzte 30 Status)
│       │   ├── AddEndpointModal.jsx     # Neuen Endpunkt hinzufügen
│       │   ├── SetIntervallModal.jsx    # Intervall konfigurieren
│       │   ├── DeleteConfirmModal.jsx   # Lösch-Bestätigung
│       │   ├── EditUrlModal.jsx         # URL bearbeiten
│       │   ├── LogModal.jsx            # Log-Tabelle mit Filter
│       │   ├── AccountSettingsModal.jsx # Passwort/E-Mail/Account löschen
│       │   └── ThemeSwitcher.jsx        # Theme-Dropdown
│       └── utils/
│           └── helpers.js              # fmtDuration, fmtInterval, normalizeUrl, mapEndpoints
└── DB/
    └── createTables.sql                # SQL-Referenz (5 Tabellen)
```
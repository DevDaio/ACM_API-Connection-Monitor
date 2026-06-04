-- ─── Benutzer-Tabelle ───
-- Speichert Account-Daten: E-Mail (eindeutig) und bcrypt-gehashtes Passwort.
CREATE TABLE IF NOT EXISTS "user" (
    userid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    emailadress VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(100) NOT NULL
);

-- ─── Endpoint-Tabelle ───
-- Speichert die zu überwachenden URLs/IPs.
-- check_type: "http" = HTTP-GET, "tcp" = TCP-Port-Check, "icmp" = ICMP-Ping
-- active: Killswitch (true = wird überwacht, false = pausiert)
CREATE TABLE IF NOT EXISTS endpoint (
    endpointid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    url VARCHAR(300) NOT NULL,
    check_type VARCHAR(10) NOT NULL DEFAULT 'http',
    active BOOLEAN NOT NULL DEFAULT true
);

-- ─── User-Endpoint-Verknuepfung ───
CREATE TABLE IF NOT EXISTS userendpoint (
    userid INTEGER NOT NULL,
    endpointid INTEGER NOT NULL,
    PRIMARY KEY (userid, endpointid),
    FOREIGN KEY (userid) REFERENCES "user"(userid) ON DELETE CASCADE,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

-- ─── Intervall-Tabelle ───
-- Konfiguriert das Check-Intervall fuer einen Endpoint (in Sekunden).
CREATE TABLE IF NOT EXISTS intervall (
    endpointid INTEGER PRIMARY KEY,
    seconds INTEGER NOT NULL,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

-- ─── Log-Tabelle ───
-- ACHTUNG: Dieses Schema wird auch automatisch in main.rs via CREATE/ALTER erstellt.
--          Dieses File dient als Referenz fuer manuelle DB-Setups.
-- status ist nullable: NULL = URL-Edit-Event, true = up, false = down.
-- url speichert die zum Zeitpunkt des Checks aktuelle URL (kann sich via Edit aendern).
-- check_type speichert welche Methode verwendet wurde: "http", "tcp", "icmp", NULL=Edit.
CREATE TABLE IF NOT EXISTS log (
    endpointid INTEGER NOT NULL,
    status BOOLEAN,
    url VARCHAR(300),
    check_type VARCHAR(10),
    statusdate DATE NOT NULL DEFAULT CURRENT_DATE,
    statustime TIME NOT NULL DEFAULT CURRENT_TIME,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

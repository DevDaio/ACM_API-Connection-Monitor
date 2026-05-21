-- ─── Benutzer-Tabelle ───
-- Speichert Account-Daten: E-Mail (eindeutig) und bcrypt-gehashtes Passwort.
-- userid wird automatisch als IDENTITY (1, 1, ...) generiert.
CREATE TABLE IF NOT EXISTS "user" (
    userid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    emailadress VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(100) NOT NULL  -- bcrypt-Hash (laenge ~60 Zeichen, daher VARCHAR(100) ausreichend)
);

-- ─── Endpoint-Tabelle ───
-- Speichert die zu überwachenden URLs.
-- Jeder Eintrag ist eine unabhaengige URL, unabhaengig vom User.
CREATE TABLE IF NOT EXISTS endpoint (
    endpointid INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    url VARCHAR(300) NOT NULL
);

-- ─── User-Endpoint-Verknuepfung ───
-- Viele-zu-Viele-Beziehung zwischen User und Endpoint.
-- Ein User kann mehrere Endpunkte haben, ein Endpunkt kann mehreren Usern gehoeren (optional).
-- PRIMARY KEY (userid, endpointid) = zusammengesetzter Schluessel (keine Duplikate).
-- ON DELETE CASCADE: Wenn ein User oder Endpoint geloescht wird, wird auch diese Verknuepfung geloescht.
CREATE TABLE IF NOT EXISTS userendpoint (
    userid INTEGER NOT NULL,
    endpointid INTEGER NOT NULL,
    PRIMARY KEY (userid, endpointid),
    FOREIGN KEY (userid) REFERENCES "user"(userid) ON DELETE CASCADE,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

-- ─── Intervall-Tabelle ───
-- Konfiguriert das Check-Intervall fuer einen Endpoint (in Sekunden).
-- 1:1-Beziehung: Jeder Endpoint hat maximal ein Intervall.
-- Kein ON DELETE CASCADE notwendig – Loeschen des Endpoints loescht das Intervall via FK.
CREATE TABLE IF NOT EXISTS intervall (
    endpointid INTEGER PRIMARY KEY,
    seconds INTEGER NOT NULL,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

-- ─── Log-Tabelle ───
-- Speichert die Ergebnisse der Monitoring-Checks.
-- Jeder Check erzeugt einen Eintrag mit Status (true=up, false=down) und Zeitstempel.
-- statusdate + statustime haben DEFAULT-Werte (aktuelles Datum/Uhrzeit bei INSERT).
-- Es gibt keinen Primaerschluessel – Logs werden rein sequentiell gespeichert.
-- Der Endpoint kann geloescht werden → alle zugehoerigen Logs werden via CASCADE entfernt.
CREATE TABLE IF NOT EXISTS log (
    endpointid INTEGER NOT NULL,
    status BOOLEAN NOT NULL,
    statusdate DATE NOT NULL DEFAULT CURRENT_DATE,
    statustime TIME NOT NULL DEFAULT CURRENT_TIME,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

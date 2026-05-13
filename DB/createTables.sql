CREATE TABLE IF NOT EXISTS "user" (
    userid INTEGER PRIMARY KEY,
    emailaddress VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(100) NOT NULL
);

CREATE TABLE IF NOT EXISTS endpoint (
    endpointid INTEGER PRIMARY KEY,
    url VARCHAR(300) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS userendpoint (
    userid INTEGER NOT NULL,
    endpointid INTEGER NOT NULL,
    PRIMARY KEY (userid, endpointid),
    FOREIGN KEY (userid) REFERENCES "user"(userid) ON DELETE CASCADE,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS intervall (
    endpointid INTEGER PRIMARY KEY,
    seconds INTEGER NOT NULL,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid)
);

CREATE TABLE IF NOT EXISTS log (
    endpointid INTEGER NOT NULL,
    status BOOLEAN NOT NULL,
    statusdate DATE NOT NULL DEFAULT CURRENT_DATE,
    statustime TIME NOT NULL DEFAULT CURRENT_TIME,
    FOREIGN KEY (endpointid) REFERENCES endpoint(endpointid)
);

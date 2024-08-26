-- tables that require foreign keys and represent entities
CREATE TABLE
    ausschuesse (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament INTEGER REFERENCES parlamente (id) ON DELETE CASCADE
    );

CREATE TABLE
    status (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament INTEGER REFERENCES parlamente (id) ON DELETE CASCADE
    );

-- updateable entities have an external uuid
CREATE TABLE
    tops (
        id SERIAL PRIMARY KEY,
        ext_id UUID UNIQUE NOT NULL,
        datum DATE NOT NULL,
        url VARCHAR(255) NOT NULL,
        parlament INTEGER REFERENCES parlamente (id) ON DELETE CASCADE
    );
CREATE TABLE
    gesetzesvorhaben (
        id SERIAL PRIMARY KEY,
        ext_id UUID UNIQUE NOT NULL,
        titel VARCHAR(255) NOT NULL,
        off_titel VARCHAR(255) NOT NULL,
        url_gesblatt VARCHAR(255),
        id_gesblatt VARCHAR(255),
        verfassungsaendernd BOOLEAN NOT NULL,
        trojaner BOOLEAN,
        federfuehrung INTEGER REFERENCES ausschuesse (id) ON DELETE SET NULL,
        initiator INTEGER REFERENCES initiatoren (id) ON DELETE SET NULL
    );

CREATE TABLE
    dokumente (
        id SERIAL PRIMARY KEY,
        ext_id UUID UNIQUE NOT NULL,
        off_id VARCHAR(255) NOT NULL,
        datum DATE NOT NULL,
        url VARCHAR(255) NOT NULL,
        collector_url VARCHAR(255) NOT NULL,
        file VARCHAR(255),
        hash CHAR(64) NOT NULL, -- TODO: check if this is the correct length
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ INTEGER REFERENCES dokumenttypen (id) ON DELETE SET NULL
    );

CREATE TABLE
    ausschussberatungen (
        id SERIAL PRIMARY KEY,
        ext_id UUID UNIQUE NOT NULL,
        datum DATE NOT NULL,
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        ausschuss INTEGER REFERENCES ausschuesse (id) ON DELETE SET NULl,
        dokument INTEGER REFERENCES dokumente (id) ON DELETE SET NULL
    );

CREATE TABLE
    sonstige_ids ( -- updated via their cascading reference
        id SERIAL PRIMARY KEY,
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ VARCHAR(255) NOT NULL,
        inhalt VARCHAR(255) NOT NULL
    );
CREATE TABLE
    abstimmungen (
        id SERIAL PRIMARY KEY, 
        ext_id UUID UNIQUE NOT NULL,
        namentlich BOOLEAN NOT NULL,
        url VARCHAR(255) NOT NULL,
        typ INTEGER REFERENCES abstimmungstyp (id) ON DELETE SET NULL,
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE
    );

CREATE TABLE
    tagesordnungspunkt ( --updated via tops
        id SERIAL PRIMARY KEY,
        titel VARCHAR(255) NOT NULL,
        tops_id INTEGER REFERENCES tops (id) ON DELETE CASCADE,
        document INTEGER REFERENCES dokumente (id) ON DELETE SET NULL,
        abstimmung INTEGER REFERENCES abstimmungen (id) ON DELETE SET NULL
    );

CREATE TABLE
    abstimmungsergebnisse ( --updated via abstimmung/tops
        id SERIAL PRIMARY KEY,
        abstimmung INTEGER REFERENCES abstimmungen (id) ON DELETE CASCADE,
        fraktion INTEGER REFERENCES fraktionen (id) ON DELETE CASCADE,
        anteil DOUBLE PRECISION NOT NULL
    );
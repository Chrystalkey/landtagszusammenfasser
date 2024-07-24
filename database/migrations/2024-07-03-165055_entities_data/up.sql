-- tables that require foreign keys and represent entities
CREATE TABLE
    ausschuesse (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament SERIAL REFERENCES parlamente (id) ON DELETE CASCADE
    );

CREATE TABLE
    tops (
        id SERIAL PRIMARY KEY,
        datum DATE NOT NULL,
        url VARCHAR(255) NOT NULL,
        parlament SERIAL REFERENCES parlamente (id) ON DELETE CASCADE
    );

CREATE TABLE
    tagesordnungspunkt (
        id SERIAL PRIMARY KEY,
        titel VARCHAR(255) NOT NULL,
        tops_id SERIAL REFERENCES tops (id) ON DELETE CASCADE,
        document SERIAL REFERENCES dokumente (id) ON DELETE SET NULL,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE SET NULL
    );



CREATE TABLE
    status (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament SERIAL REFERENCES parlamente (id) ON DELETE CASCADE
    );

CREATE TABLE
    gesetzesvorhaben (
        id UUID PRIMARY KEY,
        titel VARCHAR(255) NOT NULL,
        off_titel VARCHAR(255) NOT NULL,
        url_gesblatt VARCHAR(255),
        id_gesblatt VARCHAR(255),
        verfassungsaendernd BOOLEAN NOT NULL,
        trojaner BOOLEAN,
        federfuehrung SERIAL REFERENCES ausschuesse (id) ON DELETE SET NULL,
        initiator SERIAL REFERENCES initiatoren (id) ON DELETE SET NULL
    );

CREATE TABLE
    dokumente (
        id SERIAL PRIMARY KEY,
        off_id VARCHAR(255) NOT NULL,
        datum DATE NOT NULL,
        url VARCHAR(255) NOT NULL,
        collector_url VARCHAR(255) NOT NULL,
        file VARCHAR(255),
        hash CHAR(64) NOT NULL, -- TODO: check if this is the correct length
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ SERIAL REFERENCES dokumenttypen (id) ON DELETE SET NULL
    );

CREATE TABLE
    ausschussberatungen (
        id SERIAL PRIMARY KEY,
        datum DATE NOT NULL,
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        ausschuss SERIAL REFERENCES ausschuesse (id) ON DELETE SET NULl,
        dokument SERIAL REFERENCES dokumente (id) ON DELETE SET NULL
    );

CREATE TABLE
    sonstige_ids (
        id SERIAL PRIMARY KEY,
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ VARCHAR(255) NOT NULL,
        inhalt VARCHAR(255) NOT NULL
    );
CREATE TABLE
    abstimmungen (
        id SERIAL PRIMARY KEY,
        namentlich BOOLEAN NOT NULL,
        url VARCHAR(255) NOT NULL,
        typ SERIAL REFERENCES abstimmungstyp (id) ON DELETE SET NULL
        gesetzesvorhabe UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE
    );
CREATE TABLE
    abstimmungsergebnisse (
        id SERIAL PRIMARY KEY,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        fraktion SERIAL REFERENCES fraktionen (id) ON DELETE CASCADE,
        anteil NUMERIC NOT NULL
    );
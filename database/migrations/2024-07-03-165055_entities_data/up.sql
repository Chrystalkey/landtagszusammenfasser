-- tables that require foreign keys and represent entities
CREATE TABLE
    ausschuss (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament INTEGER NOT NULL REFERENCES parlament (id) ON DELETE CASCADE
    );

-- updateable entities have an external uuid

CREATE TABLE
    gesetzesvorhaben (
        id SERIAL PRIMARY KEY,
        api_id UUID UNIQUE NOT NULL,
        titel VARCHAR(255) NOT NULL,
        initiator VARCHAR(128) NOT NULL,
        verfassungsaendernd BOOLEAN NOT NULL,
        trojaner BOOLEAN NOT NULL,
        typ INTEGER NOT NULL REFERENCES gesetzestyp (id) ON DELETE CASCADE,
        federf INTEGER REFERENCES ausschuss (id) ON DELETE SET NULL
    );
    
CREATE TABLE station(
    id SERIAL PRIMARY KEY,
    gesetzesvorhaben INTEGER NOT NULL REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
    status INTEGER NOT NULL REFERENCES status (id) ON DELETE CASCADE,
    parlament INTEGER NOT NULL REFERENCES parlament (id) ON DELETE CASCADE,

    api_id UUID UNIQUE NOT NULL,
    datum TIMESTAMP NOT NULL,
    ausschuss INTEGER REFERENCES ausschuss (id) ON DELETE SET NULL,
    meinungstendenz INTEGER
);

CREATE TABLE rel_station_schlagwort(
    station INTEGER NOT NULL REFERENCES station (id) ON DELETE CASCADE,
    schlagwort INTEGER NOT NULL REFERENCES schlagwort (id) ON DELETE CASCADE,
    PRIMARY KEY (station, schlagwort)
);

CREATE TABLE
    dokument (
        id SERIAL PRIMARY KEY,
        api_id UUID UNIQUE NOT NULL,
        identifikator VARCHAR(255) NOT NULL,
        last_access TIMESTAMP NOT NULL,
        url VARCHAR(255) NOT NULL,
        hash CHAR(128) NOT NULL, -- TODO: check if this is the correct length
        doktyp INTEGER NOT NULL REFERENCES dokumententyp (id),
        gesetzesvorhaben INTEGER NOT NULL REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        station INTEGER NOT NULL REFERENCES station (id) ON DELETE CASCADE
    );
CREATE TABLE rel_dok_autor(
    dokument INTEGER NOT NULL REFERENCES dokument (id) ON DELETE CASCADE,
    autor INTEGER NOT NULL REFERENCES autor (id) ON DELETE CASCADE,
    PRIMARY KEY (dokument, autor)
);

CREATE TABLE further_links(
    id SERIAL PRIMARY KEY,
    link VARCHAR(255) NOT NULL,
    gesetzesvorhaben INTEGER NOT NULL REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE
);
CREATE TABLE further_notes(
    id SERIAL PRIMARY KEY,
    notes VARCHAR(255) NOT NULL,
    gesetzesvorhaben INTEGER NOT NULL REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE
);
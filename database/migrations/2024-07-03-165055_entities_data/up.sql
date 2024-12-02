CREATE TABLE dokument (
    id SERIAL PRIMARY KEY,
    titel VARCHAR NOT NULL,
    zeitpunkt TIMESTAMP NOT NULL,
    url VARCHAR NOT NULL,
    hash VARCHAR NOT NULL,
    zusammenfassung VARCHAR ,
    dokumententyp INTEGER NOT NULL REFERENCES dokumententyp(id)
);

CREATE TABLE rel_dok_autor(
    dokument_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    autor VARCHAR NOT NULL,
    PRIMARY KEY (dokument_id, autor)
);

CREATE TABLE rel_dok_schlagwort(
    dokument_id INTEGER NOT NULL REFERENCES dokument(id),
    schlagwort_id INTEGER NOT NULL REFERENCES schlagwort(id),
    PRIMARY KEY (dokument_id, schlagwort_id)
);

CREATE TABLE gesetzesvorhaben(
    id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL,
    titel VARCHAR NOT NULL,
    verfassungsaendernd BOOLEAN NOT NULL,
    typ INTEGER NOT NULL REFERENCES gesetzestyp(id)
);
CREATE TABLE rel_gesvh_init(
    gesetzesvorhaben INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (gesetzesvorhaben, initiator)
);

CREATE TABLE rel_gesvh_id (
    id SERIAL PRIMARY KEY,
    gesetzesvorhaben_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    id_typ INTEGER NOT NULL REFERENCES identifikatortyp(id),
    identifikator VARCHAR NOT NULL
);

CREATE TABLE rel_gesvh_links(
    id SERIAL PRIMARY KEY,
    gesetzesvorhaben_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    link VARCHAR NOT NULL,
    CONSTRAINT rel_gesvh_links_unique_combo UNIQUE (gesetzesvorhaben_id, link)
);

CREATE TABLE station (
    id SERIAL PRIMARY KEY,
    gesvh_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    parlament INTEGER NOT NULL REFERENCES parlament(id) ON DELETE CASCADE,
    stationstyp INTEGER NOT NULL REFERENCES stationstyp(id) ON DELETE CASCADE,
    gremium VARCHAR NOT NULL,
    zeitpunkt TIMESTAMP NOT NULL,
    trojaner BOOLEAN NOT NULL,
    url VARCHAR
);

CREATE TABLE rel_station_dokument(
    station_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    dokument_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    PRIMARY KEY (station_id, dokument_id)
);

CREATE TABLE rel_station_schlagwort(
    station_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    schlagwort_id INTEGER NOT NULL REFERENCES schlagwort(id) ON DELETE CASCADE,
    PRIMARY KEY (station_id, schlagwort_id)
);

CREATE TABLE stellungnahme (
    id SERIAL PRIMARY KEY,
    station_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    dokument_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    meinung INTEGER NOT NULL,
    lobbyregister VARCHAR
);
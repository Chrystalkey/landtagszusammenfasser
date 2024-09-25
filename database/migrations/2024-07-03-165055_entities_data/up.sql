CREATE TABLE dokument (
    id SERIAL PRIMARY KEY,
    titel VARCHAR NOT NULL,
    dokumenttyp_id INTEGER NOT NULL REFERENCES dokumenttyp(id),
    url VARCHAR NOT NULL,
    hash VARCHAR NOT NULL,
    zusammenfassung VARCHAR NOT NULL
);

CREATE TABLE rel_dok_autor(
    dokument_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    autor_id INTEGER NOT NULL REFERENCES autor(id),
    PRIMARY KEY (dokument_id, autor_id)
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
    trojaner BOOLEAN NOT NULL,
    initiative VARCHAR NOT NULL,
    typ INTEGER NOT NULL REFERENCES gesetzestyp(id)
);
CREATE TABLE rel_gesvh_id(
    gesetzesvorhaben_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id),
    id_typ INTEGER NOT NULL REFERENCES identifikatortyp(id),
    identifikator VARCHAR NOT NULL,
    PRIMARY KEY (gesetzesvorhaben_id, id_typ, identifikator)
);
CREATE TABLE rel_gesvh_links(
    id SERIAL PRIMARY KEY,
    gesetzesvorhaben_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id),
    link VARCHAR NOT NULL
);
CREATE TABLE rel_gesvh_notes(
    id SERIAL PRIMARY KEY,
    gesetzesvorhaben_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id),
    note VARCHAR NOT NULL
);

CREATE TABLE station(
    id SERIAL PRIMARY KEY,
    gesvh_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    parlament INTEGER NOT NULL REFERENCES parlament(id) ON DELETE CASCADE,
    stationstyp INTEGER NOT NULL REFERENCES stationstyp(id),
    zeitpunkt TIMESTAMP NOT NULL,
    url VARCHAR,
    zuordnung VARCHAR NOT NULL
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
    titel VARCHAR NOT NULL,
    station_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    dokument_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    zeitpunkt TIMESTAMP NOT NULL,
    meinung INTEGER NOT NULL,
    url VARCHAR NOT NULL
);

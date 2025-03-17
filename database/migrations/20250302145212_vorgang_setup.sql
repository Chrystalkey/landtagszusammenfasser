-- Add migration script here
CREATE TABLE gremium(
    id SERIAL PRIMARY KEY,
    parl INTEGER NOT NULL REFERENCES parlament(id),
    name VARCHAR NOT NULL,
    wp INTEGER NOT NULL,
    link VARCHAR,
    link_kalender VARCHAR,

    CONSTRAINT unique_combo UNIQUE (parl, name, wp)
);

CREATE TABLE dokument (
    id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    drucksnr VARCHAR,
    typ INTEGER NOT NULL REFERENCES dokumententyp(id),
    
    titel VARCHAR NOT NULL,
    kurztitel VARCHAR,
    vorwort VARCHAR,
    volltext VARCHAR NOT NULL,
    zusammenfassung VARCHAR,

    last_mod TIMESTAMP WITH TIME ZONE NOT NULL,
    link VARCHAR NOT NULL,
    hash VARCHAR NOT NULL
);

CREATE TABLE rel_dok_autor(
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    autor VARCHAR NOT NULL,
    PRIMARY KEY (dok_id, autor)
);

CREATE TABLE rel_dok_autorperson(
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    autor VARCHAR NOT NULL,
    PRIMARY KEY (dok_id, autor)
);

CREATE TABLE rel_dok_schlagwort(
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    sw_id INTEGER NOT NULL REFERENCES schlagwort(id) ON DELETE CASCADE,
    PRIMARY KEY (dok_id, sw_id)
);

CREATE TABLE vorgang (
    id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    titel VARCHAR NOT NULL,
    kurztitel VARCHAR,
    verfaend BOOLEAN NOT NULL,
    wahlperiode INTEGER NOT NULL,
    typ INTEGER NOT NULL REFERENCES vorgangstyp(id) ON DELETE CASCADE
);
CREATE TABLE rel_vorgang_init(
    vg_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (vg_id, initiator)
);

CREATE TABLE rel_vorgang_init_person(
    vg_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (vg_id, initiator)
);

CREATE TABLE rel_vg_ident (
    vg_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    typ INTEGER NOT NULL REFERENCES vg_ident_typ(id) ON DELETE CASCADE,
    identifikator VARCHAR NOT NULL,
    PRIMARY KEY (vg_id, typ, identifikator)
);

CREATE TABLE rel_vorgang_links(
    vg_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    link VARCHAR NOT NULL,
    PRIMARY KEY (vg_id, link)
);

CREATE TABLE station (
    id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    gr_id INTEGER REFERENCES gremium(id) ON DELETE SET NULL,
    gremium_isff BOOLEAN,
    vg_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    p_id INTEGER NOT NULL REFERENCES parlament(id) ON DELETE CASCADE,
    typ INTEGER NOT NULL REFERENCES stationstyp(id) ON DELETE CASCADE,
    titel VARCHAR,

    start_zeitpunkt TIMESTAMP WITH TIME ZONE NOT NULL,
    letztes_update TIMESTAMP WITH TIME ZONE NOT NULL,
    trojanergefahr INTEGER,
    link VARCHAR
);
CREATE TABLE rel_station_dokument(
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    PRIMARY KEY (stat_id, dok_id)
);

CREATE TABLE rel_station_schlagwort(
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    sw_id INTEGER NOT NULL REFERENCES schlagwort(id) ON DELETE CASCADE,
    PRIMARY KEY (stat_id, sw_id)
);
CREATE TABLE rel_station_gesetz (
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    gesetz VARCHAR NOT NULL,
    PRIMARY KEY (stat_id, gesetz)
);
CREATE TABLE rel_station_link (
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    link VARCHAR NOT NULL,
    PRIMARY KEY (stat_id, link)
);

CREATE TABLE stellungnahme (
    id SERIAL PRIMARY KEY,
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE, -- this is whatever this relates to
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE, -- this is the content
    meinung INTEGER NOT NULL,
    lobbyreg_link VARCHAR
);
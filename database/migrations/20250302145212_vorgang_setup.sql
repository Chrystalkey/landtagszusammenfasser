-- Add migration script here
CREATE TABLE gremium(
    id SERIAL PRIMARY KEY,
    parl INTEGER NOT NULL REFERENCES parlament(id),
    name VARCHAR NOT NULL,
    wp INTEGER NOT NULL,
    link VARCHAR,

    CONSTRAINT unique_combo UNIQUE (parl, name, wp)
);

CREATE TABLE autor(
    id SERIAL PRIMARY KEY,
    person VARCHAR,
    organisation VARCHAR NOT NULL,
    lobbyregister VARCHAR,
    CONSTRAINT unq_data UNIQUE(person, organisation)
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

    zp_lastmod TIMESTAMP WITH TIME ZONE NOT NULL,
    zp_referenz TIMESTAMP WITH TIME ZONE NOT NULL,
    zp_created TIMESTAMP WITH TIME ZONE NOT NULL,

    link VARCHAR NOT NULL,
    hash VARCHAR NOT NULL,
    meinung INTEGER
);

CREATE TABLE rel_dok_autor(
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    aut_id INTEGER NOT NULL REFERENCES autor(id) ON DELETE CASCADE,
    PRIMARY KEY (dok_id, aut_id)
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
    wahlperiode INTEGER NOT NULL,
    verfaend BOOLEAN NOT NULL,
    typ INTEGER NOT NULL REFERENCES vorgangstyp(id) ON DELETE CASCADE
);
CREATE TABLE rel_vorgang_init(
    vg_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    in_id INTEGER NOT NULL REFERENCES autor(id) ON DELETE CASCADE,
    PRIMARY KEY (vg_id, in_id)
);

CREATE TABLE rel_vorgang_ident (
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
    titel VARCHAR,
    zp_start TIMESTAMP WITH TIME ZONE NOT NULL,
    zp_modifiziert TIMESTAMP WITH TIME ZONE NOT NULL,

    link VARCHAR,
    trojanergefahr INTEGER,

    p_id INTEGER NOT NULL REFERENCES parlament(id) ON DELETE CASCADE,

    gr_id INTEGER REFERENCES gremium(id) ON DELETE SET NULL,
    gremium_isff BOOLEAN,
    typ INTEGER NOT NULL REFERENCES stationstyp(id) ON DELETE CASCADE,

    vg_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE
);
CREATE TABLE rel_station_schlagwort(
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    sw_id INTEGER NOT NULL REFERENCES schlagwort(id) ON DELETE CASCADE,
    PRIMARY KEY (stat_id, sw_id)
);
CREATE TABLE rel_station_dokument(
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    PRIMARY KEY (stat_id, dok_id)
);
CREATE TABLE rel_station_link (
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    link VARCHAR NOT NULL,
    PRIMARY KEY (stat_id, link)
);
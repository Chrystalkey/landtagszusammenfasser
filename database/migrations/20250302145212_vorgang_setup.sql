-- Add migration script here
CREATE TABLE gremium(
    gr_id SERIAL PRIMARY KEY,
    p_id INTEGER NOT NULL REFERENCES parlament(p_id),
    name VARCHAR NOT NULL
);

CREATE TABLE dokument (
    dok_id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    drucksnr VARCHAR,
    dtyp_id INTEGER NOT NULL REFERENCES dokumententyp(dtyp_id),
    
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
    dok_id INTEGER NOT NULL REFERENCES dokument(dok_id) ON DELETE CASCADE,
    autor VARCHAR NOT NULL,
    PRIMARY KEY (dok_id, autor)
);

CREATE TABLE rel_dok_autorperson(
    dok_id INTEGER NOT NULL REFERENCES dokument(dok_id) ON DELETE CASCADE,
    autor VARCHAR NOT NULL,
    PRIMARY KEY (dok_id, autor)
);

CREATE TABLE rel_dok_schlagwort(
    dok_id INTEGER NOT NULL REFERENCES dokument(dok_id) ON DELETE CASCADE,
    sw_id INTEGER NOT NULL REFERENCES schlagwort(sw_id) ON DELETE CASCADE,
    PRIMARY KEY (dok_id, sw_id)
);

CREATE TABLE vorgang (
    vg_id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    titel VARCHAR NOT NULL,
    kurztitel VARCHAR,
    verfaend BOOLEAN NOT NULL,
    wahlperiode INTEGER NOT NULL,
    vgtyp_id INTEGER NOT NULL REFERENCES vorgangstyp(vgtyp_id) ON DELETE CASCADE,
    CONSTRAINT unique_vorgang_titel_typ UNIQUE (titel, vgtyp_id, wahlperiode)
);
CREATE TABLE rel_vorgang_init(
    vg_id INTEGER NOT NULL REFERENCES vorgang(vg_id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (vg_id, initiator)
);

CREATE TABLE rel_vorgang_init_person(
    vg_id INTEGER NOT NULL REFERENCES vorgang(vg_id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (vg_id, initiator)
);

CREATE TABLE rel_vg_ident (
    vg_id INTEGER NOT NULL REFERENCES vorgang(vg_id) ON DELETE CASCADE,
    vgit_id INTEGER NOT NULL REFERENCES vg_ident_typ(vgit_id) ON DELETE CASCADE,
    identifikator VARCHAR NOT NULL,
    PRIMARY KEY (vg_id, vgit_id, identifikator)
);

CREATE TABLE rel_vorgang_links(
    vg_id INTEGER NOT NULL REFERENCES vorgang(vg_id) ON DELETE CASCADE,
    link VARCHAR NOT NULL,
    CONSTRAINT rel_vorgang_links_unique_combo UNIQUE (vg_id, link)
);

CREATE TABLE station (
    stat_id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    gr_id INTEGER REFERENCES gremium(gr_id) ON DELETE SET NULL,
    vg_id INTEGER NOT NULL REFERENCES vorgang(vg_id) ON DELETE CASCADE,
    p_id INTEGER NOT NULL REFERENCES parlament(p_id) ON DELETE CASCADE,
    styp_id INTEGER NOT NULL REFERENCES stationstyp(styp_id) ON DELETE CASCADE,
    titel VARCHAR,

    start_zeitpunkt TIMESTAMP WITH TIME ZONE NOT NULL,
    letztes_update TIMESTAMP WITH TIME ZONE,
    trojanergefahr INTEGER,
    link VARCHAR
);
CREATE TABLE rel_station_dokument(
    stat_id INTEGER NOT NULL REFERENCES station(stat_id) ON DELETE CASCADE,
    dok_id INTEGER NOT NULL REFERENCES dokument(dok_id) ON DELETE CASCADE,
    PRIMARY KEY (stat_id, dok_id)
);

CREATE TABLE rel_station_schlagwort(
    stat_id INTEGER NOT NULL REFERENCES station(stat_id) ON DELETE CASCADE,
    sw_id INTEGER NOT NULL REFERENCES schlagwort(sw_id) ON DELETE CASCADE,
    PRIMARY KEY (stat_id, sw_id)
);

CREATE TABLE rel_station_gesetz (
    stat_id INTEGER NOT NULL REFERENCES station(stat_id) ON DELETE CASCADE,
    gesetz VARCHAR NOT NULL,
    PRIMARY KEY (stat_id, gesetz)
);

CREATE TABLE stellungnahme (
    stl_id SERIAL PRIMARY KEY,
    stat_id INTEGER NOT NULL REFERENCES station(stat_id) ON DELETE CASCADE, -- this is whatever this relates to
    dok_id INTEGER NOT NULL REFERENCES dokument(dok_id) ON DELETE CASCADE, -- this is the content
    meinung INTEGER NOT NULL,
    lobbyreg_link VARCHAR
);
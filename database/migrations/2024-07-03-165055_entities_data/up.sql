CREATE TABLE ausschuss(
    id SERIAL PRIMARY KEY,
    parl_id INTEGER NOT NULL REFERENCES parlament(id),
    name VARCHAR NOT NULL
);

CREATE TABLE dokument (
    id SERIAL PRIMARY KEY,
    titel VARCHAR NOT NULL,
    last_mod TIMESTAMP WITH TIME ZONE NOT NULL,
    volltext VARCHAR,
    link VARCHAR NOT NULL,
    hash VARCHAR NOT NULL,
    zusammenfassung VARCHAR,
    typ INTEGER NOT NULL REFERENCES dokumententyp(id),
    drucksnr VARCHAR
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
    verfaend BOOLEAN NOT NULL,
    wahlperiode INTEGER NOT NULL,
    typ INTEGER NOT NULL REFERENCES vorgangstyp(id) ON DELETE CASCADE,
    CONSTRAINT unique_vorgang_titel_typ UNIQUE (titel, typ, wahlperiode)
);
CREATE TABLE rel_vorgang_init(
    vorgang_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (vorgang_id, initiator)
);

CREATE TABLE rel_vorgang_init_person(
    vorgang_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (vorgang_id, initiator)
);

CREATE TABLE rel_vorgang_id (
    vorgang_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    typ INTEGER NOT NULL REFERENCES identifikatortyp(id) ON DELETE CASCADE,
    identifikator VARCHAR NOT NULL,
    PRIMARY KEY (vorgang_id, typ, identifikator)
); 

CREATE TABLE rel_vorgang_links(
    id SERIAL PRIMARY KEY,
    vorgang_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    link VARCHAR NOT NULL,
    CONSTRAINT rel_vorgang_links_unique_combo UNIQUE (vorgang_id, link)
);
CREATE TABLE top(
    id SERIAL PRIMARY KEY,
    vorgang_id INTEGER REFERENCES vorgang(id),
    titel VARCHAR NOT NULL,
    number INTEGER NOT NULL
);
CREATE TABLE tops_drucks(
    top_id INTEGER NOT NULL REFERENCES top(id) ON DELETE CASCADE,
    drucks_nr VARCHAR NOT NULL,
    PRIMARY KEY (top_id, drucks_nr)
);

CREATE TABLE ausschusssitzung(
    id SERIAL PRIMARY KEY,
    termin TIMESTAMP WITH TIME ZONE NOT NULL,
    public BOOLEAN NOT NULL,
    as_id INTEGER NOT NULL REFERENCES ausschuss(id) ON DELETE CASCADE
);
CREATE TABLE rel_ass_experten(
    ass_id INTEGER NOT NULL REFERENCES ausschusssitzung(id) ON DELETE CASCADE,
    exp_id INTEGER NOT NULL REFERENCES experte(id) ON DELETE CASCADE,
    PRIMARY KEY (ass_id, exp_id)
);
CREATE TABLE rel_ass_tops(
    ass_id INTEGER NOT NULL REFERENCES ausschusssitzung(id) ON DELETE CASCADE,
    top_id INTEGER NOT NULL REFERENCES top(id) ON DELETE CASCADE,
    PRIMARY KEY (ass_id, top_id)
);

CREATE TABLE station (
    id SERIAL PRIMARY KEY,
    vorgang_id INTEGER NOT NULL REFERENCES vorgang(id) ON DELETE CASCADE,
    parl_id INTEGER NOT NULL REFERENCES parlament(id) ON DELETE CASCADE,
    typ INTEGER NOT NULL REFERENCES stationstyp(id) ON DELETE CASCADE,
    titel VARCHAR,

    zeitpunkt TIMESTAMP WITH TIME ZONE,
    trojanergefahr INTEGER,
    link VARCHAR
);
CREATE TABLE rel_station_ausschusssitzung(
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE  CASCADE,
    as_id INTEGER NOT NULL REFERENCES ausschusssitzung(id) ON DELETE CASCADE,
    PRIMARY KEY (stat_id, as_id)
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

CREATE TABLE stellungnahme (
    id SERIAL PRIMARY KEY,
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE, -- this is whatever this relates to
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE, -- this is the content
    meinung INTEGER NOT NULL,
    lobbyreg_link VARCHAR,
    volltext VARCHAR
);

--- trigger & function to delete orphaned dokument from station reference
CREATE OR REPLACE FUNCTION delete_orphaned_dokument_station() 
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM dokument
    WHERE id = OLD.dok_id
      AND NOT EXISTS (SELECT 1 FROM rel_station_dokument WHERE dok_id = OLD.dok_id);
    RETURN OLD;
END;
$$
 LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_orphaned_dokument_station
AFTER DELETE ON rel_station_dokument
FOR EACH ROW EXECUTE FUNCTION delete_orphaned_dokument_station();

--- trigger & function to delete orphaned dokument from "stellungnahme" reference
CREATE OR REPLACE FUNCTION delete_orphaned_dokument_stellungnahme() 
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM dokument
    WHERE id = OLD.dok_id
      AND NOT EXISTS (SELECT 1 FROM stellungnahme WHERE dok_id = OLD.dok_id);
    RETURN OLD;
END;
$$
 LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_orphaned_dokument_stellungnahme
AFTER DELETE ON stellungnahme
FOR EACH ROW EXECUTE FUNCTION delete_orphaned_dokument_stellungnahme();
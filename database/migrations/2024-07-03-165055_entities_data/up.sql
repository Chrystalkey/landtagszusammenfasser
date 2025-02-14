CREATE TABLE dokument (
    id SERIAL PRIMARY KEY,
    titel VARCHAR NOT NULL,
    datum TIMESTAMP WITH TIME ZONE NOT NULL,
    link VARCHAR NOT NULL,
    hash VARCHAR NOT NULL,
    zusammenfassung VARCHAR,
    typ INTEGER NOT NULL REFERENCES dokumententyp(id)
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

CREATE TABLE gesetzesvorhaben(
    id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    titel VARCHAR NOT NULL,
    verfaend BOOLEAN NOT NULL,
    typ INTEGER NOT NULL REFERENCES gesetzestyp(id) ON DELETE CASCADE,
    CONSTRAINT unique_gsvh_titel_typ UNIQUE (titel, typ)
);
CREATE TABLE rel_gsvh_init(
    gsvh_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (gsvh_id, initiator)
);

CREATE TABLE rel_gsvh_init_person(
    gsvh_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    initiator VARCHAR NOT NULL,
    PRIMARY KEY (gsvh_id, initiator)
);

CREATE TABLE rel_gsvh_id (
    gsvh_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    typ INTEGER NOT NULL REFERENCES identifikatortyp(id) ON DELETE CASCADE,
    identifikator VARCHAR NOT NULL,
    PRIMARY KEY (gsvh_id, typ, identifikator)
); 

CREATE TABLE rel_gsvh_links(
    id SERIAL PRIMARY KEY,
    gsvh_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    link VARCHAR NOT NULL,
    CONSTRAINT rel_gsvh_links_unique_combo UNIQUE (gsvh_id, link)
);

CREATE TABLE station (
    id SERIAL PRIMARY KEY,
    gsvh_id INTEGER NOT NULL REFERENCES gesetzesvorhaben(id) ON DELETE CASCADE,
    parl_id INTEGER NOT NULL REFERENCES parlament(id) ON DELETE CASCADE,
    typ INTEGER NOT NULL REFERENCES stationstyp(id) ON DELETE CASCADE,
    gremium VARCHAR NOT NULL,
    datum TIMESTAMP WITH TIME ZONE NOT NULL,
    trojaner BOOLEAN NOT NULL,
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

CREATE TABLE stellungnahme (
    id SERIAL PRIMARY KEY,
    stat_id INTEGER NOT NULL REFERENCES station(id) ON DELETE CASCADE,
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    meinung INTEGER,
    lobbyreg_link VARCHAR
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
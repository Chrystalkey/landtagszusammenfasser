-- tables that do not require any foreign keys
CREATE TABLE
    vorgangstyp (
        id SERIAL PRIMARY KEY,
        value VARCHAR NOT NULL
    );

CREATE TABLE
    vg_ident_typ (
        id SERIAL PRIMARY KEY,
        value VARCHAR NOT NULL
    );

CREATE TABLE
    parlament (
        id SERIAL PRIMARY KEY,
        value VARCHAR NOT NULL
    );

CREATE TABLE
    schlagwort (
        id SERIAL PRIMARY KEY,
        value VARCHAR UNIQUE NOT NULL
    );

CREATE TABLE
    dokumententyp (
        id SERIAL PRIMARY KEY,
        value VARCHAR NOT NULL
    );

CREATE TABLE
    stationstyp (
        id SERIAL PRIMARY KEY,
        value VARCHAR NOT NULL
    );

CREATE TABLE experte(
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    fachgebiet VARCHAR NOT NULL
);

INSERT INTO
    vorgangstyp (value)
VALUES
    ('gg-einspruch'),
    ('gg-zustimmung'),
    ('gg-land-parl'),
    ('gg-land-volk'),
    ('sonstig');

INSERT INTO
    vg_ident_typ (value)
VALUES
    ('initdrucks'),
    ('vorgnr'),
    ('api-id'),
    ('sonstig');

INSERT INTO
    stationstyp (value)
VALUES
    ('preparl-regent'),
    ('preparl-eckpup'),
    ('preparl-regbsl'),
    ('preparl-vbegde'),
    ('parl-initiativ'),
    ('parl-ausschber'),
    ('parl-vollvlsgn'),
    ('parl-akzeptanz'),
    ('parl-ablehnung'),
    ('parl-zurueckgz'),
    ('parl-ggentwurf'),
    ('postparl-vesja'),
    ('postparl-vesne'),
    ('postparl-gsblt'),
    ('postparl-kraft'),
    ('sonstig');

INSERT INTO parlament(value) VALUES 
('BT'), ('BR'), ('BV'), ('EK'), 
('BB'), ('BY'), ('BE'), ('HB'), 
('HH'), ('HE'), ('MV'), ('NI'), 
('NW'), ('RP'), ('SL'), ('SN'), 
('SH'), ('TH'), ('BW'), ('ST');

INSERT INTO
    dokumententyp (value)
VALUES
    ('preparl-entwurf'),
    ('entwurf'),
    ('mitteilung'),
    ('stellungnahme'),
    ('gutachten'),
    ('beschlussempf'),
    ('plenar-protokoll'),
    ('plenar-tops'),
    ('as-tops'),
    ('as-tops-aend'),
    ('as-tops-ergz'),
    ('sonstig');

CREATE EXTENSION pg_trgm;
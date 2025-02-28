-- Your SQL goes here
INSERT INTO
    vorgangstyp (api_key)
VALUES
    ('gg-einspruch'),
    ('gg-zustimmung'),
    ('gg-land-parl'),
    ('gg-land-volk'),
    ('sonstig');

INSERT INTO
    vg_ident_typ (api_key)
VALUES
    ('initdrucks'),
    ('vorgnr'),
    ('api-id'),
    ('sonstig');

INSERT INTO
    stationstyp (api_key)
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
    ('parl-ggentwurf'),
    ('postparl-vesja'),
    ('postparl-vesne'),
    ('postparl-gsblt'),
    ('postparl-kraft'),
    ('sonstig');

INSERT INTO parlament(api_key) VALUES 
('BT'), ('BR'), ('BV'), ('EK'), 
('BB'), ('BY'), ('BE'), ('HB'), 
('HH'), ('HE'), ('MV'), ('NI'), 
('NW'), ('RP'), ('SL'), ('SN'), 
('SH'), ('TH'), ('BW'), ('ST');

INSERT INTO
    dokumententyp (api_key)
VALUES
    ('entwurf'),
    ('drucksache'),
    ('protokoll'),
    ('topliste'),
    ('stellungnahme'),
    ('sonstig');

CREATE EXTENSION pg_trgm;
-- Your SQL goes here
INSERT INTO
    gesetzestyp (api_key)
VALUES
    ('bgg-einspruch'),
    ('bgg-zustimmung'),
    ('bay-parlament'),
    ('bay-volk'),
    ('sonstig');

INSERT INTO
    identifikatortyp (api_key)
VALUES
    ('initdrucks'),
    ('vorgnr'),
    ('sonstig');

INSERT INTO
    stationstyp (api_key)
VALUES
    ('preparl-regent'),
    ('preparl-eckpup'),
    ('preparl-kabbsl'),
    ('preparl-vbegde'),
    ('parl-initiativ'),
    ('parl-ausschber'),
    ('parl-vollvlsgn'),
    ('parl-akzeptanz'),
    ('parl-ablehnung'),
    ('parl-ggentwurf'),
    ('postparl-vents'),
    ('postparl-gsblt'),
    ('postparl-kraft');

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
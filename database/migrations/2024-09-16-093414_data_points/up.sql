-- Your SQL goes here
INSERT INTO
    gesetzestyp (value)
VALUES
    ('Zustimmungsgesetz'),
    ('Einspruchsgesetz'),
    ('Volksbegehren'),
    ('Standard'),
    ('Sonstig');

REVOKE ALL ON TABLE gesetzestyp
FROM
    public;

GRANT
SELECT
    ON TABLE gesetzestyp TO public;

INSERT INTO
    identifikatortyp (value)
VALUES
    ('Vorgangsnummer'),
    ('Drucksachennummer');

REVOKE ALL ON TABLE identifikatortyp
FROM
    public;

GRANT
SELECT
    ON TABLE identifikatortyp TO public;

INSERT INTO
    stationstyp (value)
VALUES
    ('EntwurfReferentenentwurf'),
    ('EntwurfEckpunktepapier'),
    ('ParlamentInitiative'),
    ('ParlamentKabinettsbeschluss'),
    ('ParlamentStellungnahme'),
    ('ParlamentBeschlussempfehlung'),
    ('ParlamentPlenarsitzung'),
    ('ParlamentBeschluss'),
    ('Inkraftgetreten'),
    ('Abgelehnt');
REVOKE ALL ON TABLE stationstyp
FROM
    public;

GRANT
SELECT
    ON TABLE stationstyp TO public;

INSERT INTO parlament(value) VALUES 
('BT'), ('BR'), ('BV'), ('EK'), 
('BB'), ('BY'), ('BE'), ('HB'), 
('HH'), ('HE'), ('MV'), ('NI'), 
('NW'), ('RP'), ('SL'), ('SN'), 
('SH'), ('TH'), ('BW'), ('ST');

REVOKE ALL ON TABLE parlament
FROM
    public;

GRANT
SELECT
    ON TABLE parlament TO public;

INSERT INTO
    dokumenttyp (value)
VALUES
    ('Protokoll'),
    ('Gesetzesentwurf'),
    ('Stellungnahme'),
    ('Beschluss'),
    ('Beschlussempfehlung'),
    ('Sonstiges');

REVOKE ALL ON TABLE dokumenttyp
FROM
    public;

GRANT
SELECT
    ON TABLE dokumenttyp TO public;
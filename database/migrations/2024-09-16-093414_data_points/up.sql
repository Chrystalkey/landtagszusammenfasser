-- Your SQL goes here
INSERT INTO
    gesetzestyp (value)
VALUES
    ('Zustimmungsgesetz'),
    ('Einspruchsgesetz'),
    ('Volksbegehren');

REVOKE ALL ON TABLE gesetzestyp
FROM
    public;

GRANT
SELECT
    ON TABLE gesetzestyp TO public;
    
INSERT INTO
    status (value)
VALUES
    ('Entwurf: Referentenentwurf'),
    ('Entwurf: Eckpunktepapier'),
    ('Parlament: Initiative'),
    ('Kabinettsbeschluss'),
    ('Parlament: Stellungnahme'),
    ('Parlament: Beschlussempfehlung'),
    ('Parlament: Lesung/Plenarsitzung'),
    ('Parlament: Beschluss'),
    ('In Kraft Getreten'),
    ('Abgelehnt');

INSERT INTO
    parlament (name, kurzname)
VALUES
    ('Bundestag', 'BT'),
    ('Bundesrat', 'BR'),
    ('Bundesversammlung', 'BV'),
    ('Baden Württemberg', 'BW'),
    ('Bayern', 'BY'),
    ('Berlin', 'BE'),
    ('Brandenburg', 'BB'),
    ('Bremen', 'HB'),
    ('Hamburg', 'HH'),
    ('Hessen', 'HE'),
    ('Mecklenburg-Vorpommern', 'MV'),
    ('Niedersachsen', 'NI'),
    ('Nordrhein-Westfalen', 'NW'),
    ('Rheinland-Pfalz', 'RP'),
    ('Saarland', 'SL'),
    ('Sachsen', 'SN'),
    ('Sachsen-Anhalt', 'ST'),
    ('Schleswig-Holstein', 'SH'),
    ('Thüringen', 'TH');

REVOKE ALL ON TABLE parlament
FROM
    public;

GRANT
SELECT
    ON TABLE parlament TO public;

INSERT INTO
    dokumententyp (value)
VALUES
    ('Protokoll'),
    ('Gesetzesentwurf'),
    ('Stellungnahme'),
    ('Beschlussfassung');

REVOKE ALL ON TABLE dokumententyp
FROM
    public;

GRANT
SELECT
    ON TABLE dokumententyp TO public;
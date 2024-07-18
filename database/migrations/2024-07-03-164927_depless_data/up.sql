-- tables that do not require any foreign keys
CREATE TABLE
    gesetzeseigenschaften (
        id SERIAL PRIMARY KEY,
        eigenschaft VARCHAR(255) NOT NULL
    );

INSERT INTO
    gesetzeseigenschaften (id, eigenschaft)
VALUES
    (1, 'Zustimmungsgesetz'),
    (2, 'Einspruchsgesetz'),
    (3, 'Volksbegehren');

CREATE TABLE
    parlamente (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        kurzname CHAR(2) NOT NULl
    );

INSERT INTO
    parlamente (id, name, kurzname)
VALUES
    (1, 'Bundestag', 'BT'),
    (2, 'Bundesrat', 'BR'),
    (3, 'Bundesversammlung', 'BV'),
    (4, 'Baden Württemberg', 'BW'),
    (5, 'Bayern', 'BY'),
    (6, 'Berlin', 'BE'),
    (7, 'Brandenburg', 'BB'),
    (8, 'Bremen', 'HB'),
    (9, 'Hamburg', 'HH'),
    (10, 'Hessen', 'HE'),
    (11, 'Mecklenburg-Vorpommern', 'MV'),
    (12, 'Niedersachsen', 'NI'),
    (13, 'Nordrhein-Westfalen', 'NW'),
    (14, 'Rheinland-Pfalz', 'RP'),
    (15, 'Saarland', 'SL'),
    (16, 'Sachsen', 'SN'),
    (17, 'Sachsen-Anhalt', 'ST'),
    (18, 'Schleswig-Holstein', 'SH'),
    (19, 'Thüringen', 'TH');

CREATE TABLE
    initiatoren (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        organisation VARCHAR(255) NOT NULL,
        url VARCHAR(255) NOT NULL
    );

CREATE TABLE
    schlagworte (
        id SERIAL PRIMARY KEY,
        schlagwort VARCHAR(255) NOT NULL,
        beschreibung VARCHAR(255) NOT NULL
    );

CREATE TABLE
    abstimmungstyp (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL);

-- actual party factions, also 'dafür', 'dagegen', 'enthalten', 'nicht abgestimmt'
CREATE TABLE
    fraktionen (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL);

CREATE TABLE
    dokumenttypen (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL);

INSERT INTO
    dokumenttypen (id, name)
VALUES
    (1, 'Eckpunktepapier'),
    (2, 'Referentenentwurf'),
    (3, 'Beschlussempfehlung'),
    (4, 'Finale Fassung'),
    (5, 'Gesetzesentwurf'),
    (6, "Kabinettsbeschluss");
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

    -- tables that require foreign keys and represent entities
CREATE TABLE
    ausschuesse (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament SERIAL REFERENCES parlamente (id) ON DELETE CASCADE
    );

CREATE TABLE
    tops (
        id SERIAL PRIMARY KEY,
        datum DATE NOT NULL,
        url VARCHAR(255) NOT NULL,
        parlament SERIAL REFERENCES parlamente (id) ON DELETE CASCADE
    );

CREATE TABLE
    tagesordnungspunkt (
        id SERIAL PRIMARY KEY,
        titel VARCHAR(255) NOT NULL,
        tops_id SERIAL REFERENCES tops (id) ON DELETE CASCADE,
        document SERIAL REFERENCES dokumente (id) ON DELETE SET NULL,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE SET NULL
    );

CREATE TABLE
    abstimmungen (
        id SERIAL PRIMARY KEY,
        namentlich BOOLEAN NOT NULL,
        url VARCHAR(255) NOT NULL,
        typ SERIAL REFERENCES abstimmungstyp (id) ON DELETE SET NULL
    );

CREATE TABLE
    status (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament SERIAL REFERENCES parlamente (id) ON DELETE CASCADE
    );

CREATE TABLE
    gesetzesvorhaben (
        id UUID PRIMARY KEY,
        titel VARCHAR(255) NOT NULL,
        off_titel VARCHAR(255) NOT NULL,
        url_gesblatt VARCHAR(255),
        id_gesblatt VARCHAR(255),
        verfassungsaendernd BOOLEAN NOT NULL,
        aktuelles_dokument SERIAL REFERENCES dokumente (id) ON DELETE SET NULL,
        trojaner BOOLEAN,
        federfuehrung SERIAL REFERENCES ausschuesse (id) ON DELETE SET NULL,
        initiator SERIAL REFERENCES initiatoren (id) ON DELETE SET NULL
    );

CREATE TABLE
    dokumente (
        id SERIAL PRIMARY KEY,
        off_id VARCHAR(255) NOT NULL,
        datum DATE NOT NULL,
        url VARCHAR(255) NOT NULL,
        collector_url VARCHAR(255) NOT NULL,
        file VARCHAR(255),
        hash CHAR(64) NOT NULL, -- TODO: check if this is the correct length
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ SERIAL REFERENCES dokumenttypen (id) ON DELETE SET NULL
    );

CREATE TABLE
    ausschussberatungen (
        id SERIAL PRIMARY KEY,
        datum DATE NOT NULL,
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        ausschuss SERIAL REFERENCES ausschuesse (id) ON DELETE SET NULl,
        dokument SERIAL REFERENCES dokumente (id) ON DELETE SET NULL
    );

CREATE TABLE
    sonstige_ids (
        id SERIAL PRIMARY KEY,
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ VARCHAR(255) NOT NULL,
        inhalt VARCHAR(255) NOT NULL
    );

CREATE TABLE
    abstimmungsergebnisse (
        id SERIAL PRIMARY KEY,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        fraktion SERIAL REFERENCES fraktionen (id) ON DELETE CASCADE,
        anteil NUMERIC NOT NULL
    );

-- tables that require foreign keys and represent relations
CREATE TABLE
    rel_ges_schlagworte (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        schlagwort SERIAL REFERENCES schlagworte (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, schlagwort)
    );

CREATE TABLE
    rel_ges_abstimmungen (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, abstimmung)
    );

CREATE TABLE
    rel_ges_eigenschaft (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        eigenschaft SERIAL REFERENCES gesetzeseigenschaften (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, eigenschaft)
    );

CREATE TABLE
    rel_ges_status (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        status SERIAL REFERENCES status (id) ON DELETE CASCADE,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        datum DATE NOT NULL,
        active BOOLEAN NOT NULL,
        PRIMARY KEY (gesetzesvorhaben, status, abstimmung)
    );

CREATE TABLE
    rel_ges_tops (
        top SERIAL REFERENCES tops (id),
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        dokument SERIAL REFERENCES dokumente (id) ON DELETE CASCADE,
        titel VARCHAR(255) NOT NULL,
        PRIMARY KEY (top, gesetzesvorhaben, dokument, abstimmung)
    );
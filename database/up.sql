-- tables that do not require any foreign keys
CREATE TABLE
    gesetzeseigenschaften (
        id SERIAL PRIMARY KEY,
        eigenschaft VARCHAR(255) NOT NULL,
    );

INSERT INTO
    gesetzeseigenschaften (id, eigenschaft)
VALUES
    (1, 'Zustimmungsgesetz'),
    (2, "Einspruchsgesetz"),
    (3, "Volksbegehren");

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
    (3, "Bundesversammlung", 'BV'),
    (4, "Baden Württemberg", "BW"),
    (5, "Bayern", "BY"),
    (6, "Berlin", "BE"),
    (7, "Brandenburg", "BB"),
    (8, "Bremen", "HB"),
    (9, "Hamburg", "HH"),
    (10, "Hessen", "HE"),
    (11, "Mecklenburg-Vorpommern", "MV"),
    (12, "Niedersachsen", "NI"),
    (13, "Nordrhein-Westfalen", "NW"),
    (14, "Rheinland-Pfalz", "RP"),
    (15, "Saarland", "SL"),
    (16, "Sachsen", "SN"),
    (17, "Sachsen-Anhalt", "ST"),
    (18, "Schleswig-Holstein", "SH"),
    (19, "Thüringen", "TH");

CREATE TABLE
    initiatoren (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        organisation VARCHAR(255) NOT NULL,
        url VARCHAR(255) NOT NULL,
    );

CREATE TABLE
    schlagworte (
        id SERIAL PRIMARY KEY,
        schlagwort VARCHAR(255) NOT NULL,
        beschreibung VARCHAR(255) NOT NULL,
    );

CREATE TABLE
    abstimmungstyp (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
    );

-- actual party factions, also "dafür", "dagegen", "enthalten", "nicht abgestimmt"
CREATE TABLE
    fraktionen (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
    );

CREATE TABLE
    dokumenttypen (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
    );

INSERT INTO
    dokumenttypen (id, name)
VALUES
    (1, "Eckpunktepapier"),
    (2, "Referentenentwurf"),
    (3, "Beschlussempfehlung"),
    (4, "Finale Fassung"),
    (5, "Gesetzesentwurf");

-- tables that require foreign keys and represent entities
CREATE TABLE
    ausschuesse (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament SERIAL FOREIGN KEY REFERENCES parlamente (id) ON DELETE CASCADE,
    );

CREATE TABLE
    tops (
        id SERIAL PRIMARY KEY,
        datum DATE NOT NULL,
        url VARCHAR(255) NOT NULL,
        parlament SERIAL FOREIGN KEY REFERENCES parlamente (id) ON DELETE CASCADE,
    );

CREATE TABLE
    abstimmungen (
        id SERIAL PRIMARY KEY,
        namentlich BOOLEAN NOT NULL,
        url VARCHAR(255) NOT NULL,
        typ SERIAL FOREIGN KEY REFERENCES abstimmungstyp (id) ON DELETE SET NULL,
    );

CREATE TABLE
    status (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        parlament SERIAL FOREIGN KEY REFERENCES parlamente (id) ON DELETE CASCADE,
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
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ SERIAL FOREIGN KEY REFERENCES dokumenttypen (id) ON DELETE SET NULL,
    );

CREATE TABLE
    ausschussberatungen (
        id SERIAL PRIMARY KEY,
        datum DATE NOT NULL,
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        ausschuss SERIAL FOREIGN KEY REFERENCES ausschuesse (id) ON DELETE SET NULl,
        dokument SERIAL FOREIGN KEY REFERENCES dokumente (id) ON DELETE SET NULL,
    );

CREATE TABLE
    gesetzesvorhaben (
        id SERIAL PRIMARY KEY,
        titel VARCHAR(255) NOT NULL,
        off_titel VARCHAR(255) NOT NULL,
        url_gesblatt VARCHAR(255) NOT NULL,
        id_gesblatt VARCHAR(255) NOT NULL,
        verfassungsaendernd BOOLEAN NOT NULL trojaner BOOLEAN,
        federfuehrung SERIAL FOREIGN KEY REFERENCES ausschuesse (id) ON DELETE SET NULL,
        initiator SERIAL FOREIGN KEY REFERENCES initiatoren (id) ON DELETE SET NULL,
    );

CREATE TABLE
    sonstige_ids (
        id SERIAL PRIMARY KEY,
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        typ VARCHAR(255) NOT NULL,
        inhalt VARCHAR(255) NOT NULL,
    );

CREATE TABLE
    abstimmungsergebnisse (
        id SERIAL PRIMARY KEY,
        abstimmung SERIAL FOREIGN KEY REFERENCES abstimmungen (id) ON DELETE CASCADE,
        fraktion SERIAL FOREIGN KEY REFERENCES fraktionen (id) ON DELETE CASCADE,
        anteil NUMERIC NOT NULL
    );

-- tables that require foreign keys and represent relations
CREATE TABLE
    rel_ges_schlagworte (
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        schlagwort SERIAL FOREIGN KEY REFERENCES schlagworte (id) ON DELETE CASCADE,
    );

CREATE TABLE
    rel_ges_abstimmungen (
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        abstimmung SERIAL FOREIGN KEY REFERENCES abstimmungen (id) ON DELETE CASCADE,
    );

CREATE TABLE
    rel_ges_eigenschaft (
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        eigenschaft SERIAL FOREIGN KEY REFERENCES gesetzeseigenschaften (id) ON DELETE CASCADE,
    );

CREATE TABLE
    rel_ges_status (
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        status SERIAL FOREIGN KEY REFERENCES status (id) ON DELETE CASCADE,
        abstimmung SERIAL FOREIGN KEY REFERENCES abstimmungen (id) ON DELETE CASCADE,
        datum DATE NOT NULL,
        active BOOLEAN NOT NULL,
    );

CREATE TABLE
    rel_ges_tops (
        titel VARCHAR(255) NOT NULL,
        top SERIAL FOREIGN KEY REFERENCES tops (id),
        gesetzesvorhaben SERIAL FOREIGN KEY REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        abstimmung SERIAL FOREIGN KEY REFERENCES abstimmungen (id) ON DELETE CASCADE,
        dokument SERIAL FOREIGN KEY REFERENCES dokumente (id) ON DELETE CASCADE,
    );
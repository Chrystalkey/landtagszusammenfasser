-- tables that do not require any foreign keys
CREATE TABLE
    gesetzeseigenschaften (
        id SERIAL PRIMARY KEY,
        eigenschaft VARCHAR(255) NOT NULL
    );

CREATE TABLE
    parlamente (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        kurzname CHAR(2) UNIQUE NOT NULL
    );

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

-- actual party factions, and also 'dafür', 'dagegen', 'enthalten', 'nicht abgestimmt'
CREATE TABLE
    fraktionen (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL);

CREATE TABLE
    dokumenttypen (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL);

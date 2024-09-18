-- tables that do not require any foreign keys
CREATE TABLE
    gesetzestyp (
        id SERIAL PRIMARY KEY,
        value VARCHAR(255) NOT NULL
    );

CREATE TABLE
    parlament (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        kurzname CHAR(2) UNIQUE NOT NULL
    );

CREATE TABLE
    autor (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        organisation VARCHAR(255) NOT NULL
    );

CREATE TABLE
    schlagwort (
        id SERIAL PRIMARY KEY,
        value VARCHAR(255) NOT NULL
    );

CREATE TABLE
    dokumententyp (id SERIAL PRIMARY KEY, value VARCHAR(255) NOT NULL);

CREATE TABLE
    status (
        id SERIAL PRIMARY KEY,
        value VARCHAR(255) NOT NULL
    );

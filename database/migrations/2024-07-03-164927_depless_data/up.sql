-- tables that do not require any foreign keys
CREATE TABLE
    gesetzestyp (
        id SERIAL PRIMARY KEY,
        api_key VARCHAR NOT NULL
    );

CREATE TABLE
    identifikatortyp (
        id SERIAL PRIMARY KEY,
        api_key VARCHAR NOT NULL
    );

CREATE TABLE
    parlament (
        id SERIAL PRIMARY KEY,
        api_key VARCHAR NOT NULL
    );

CREATE TABLE
    schlagwort (
        id SERIAL PRIMARY KEY,
        api_key VARCHAR UNIQUE NOT NULL
    );

CREATE TABLE
    dokumententyp (
        id SERIAL PRIMARY KEY,
        api_key VARCHAR NOT NULL
    );

CREATE TABLE
    stationstyp (
        id SERIAL PRIMARY KEY,
        api_key VARCHAR NOT NULL
    );
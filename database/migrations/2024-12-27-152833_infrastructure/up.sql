-- Your SQL goes here

CREATE TABLE dokument_versions(
    dok_id INTEGER REFERENCES dokument(id) ON DELETE CASCADE,
    previous_id INTEGER REFERENCES dokument(id)  ON DELETE CASCADE,
    PRIMARY KEY(dok_id, previous_id)
);

CREATE TABLE api_scope(
    id INTEGER PRIMARY KEY,
    api_key VARCHAR NOT NULL UNIQUE
);

INSERT INTO api_scope(id, api_key) 
VALUES
(1, 'admin'),
(2, 'collector'),
(3, 'keyadder');

CREATE TABLE api_keys (
    id INTEGER PRIMARY KEY,
    key_hash VARCHAR NOT NULL,
    coll_id UUID UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by INTEGER NOT NULL REFERENCES api_keys,
    last_used TIMESTAMP,
    scope INTEGER REFERENCES api_scope,
    deleted BOOL NOT NULL DEFAULT false
);
-- Your SQL goes here

CREATE TABLE dokument_versions(
    dok_id INTEGER REFERENCES dokument(id) ON DELETE CASCADE,
    previous_id INTEGER REFERENCES dokument(id)  ON DELETE CASCADE,
    PRIMARY KEY(dok_id, previous_id)
);

CREATE TABLE api_keys (
    id INTEGER PRIMARY KEY,
    coll_id UUID NOT NULL UNIQUE,
    key_hash VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP,
    deleted BOOL NOT NULL DEFAULT false
);

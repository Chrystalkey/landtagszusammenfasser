-- Your SQL goes here

CREATE TABLE dokument_versions(
    dok_id INTEGER REFERENCES dokument(id) ON DELETE CASCADE,
    previous_id INTEGER REFERENCES dokument(id)  ON DELETE CASCADE,
    PRIMARY KEY(dok_id, previous_id)
);

CREATE TABLE collector_auth(
    id INTEGER PRIMARY KEY,
    coll_id UUID NOT NULL UNIQUE,
    pubkey VARCHAR NOT NULL UNIQUE,
    deleted BOOL NOT NULL DEFAULT false
);

CREATE TABLE ip_lastreq(
    ip INTEGER PRIMARY KEY,
    request_ts TIMESTAMP NOT NULL
);
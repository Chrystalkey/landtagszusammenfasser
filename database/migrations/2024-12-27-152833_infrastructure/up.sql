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
(1, 'admin'),     -- directly inserting, updating and deleting laws
(2, 'collector'), -- "normal" insertion of laws
(3, 'keyadder');  -- adding new api keys

CREATE TABLE api_keys (
    id SERIAL PRIMARY KEY,
    key_hash VARCHAR NOT NULL,
    coll_id UUID UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT CURRENT_TIMESTAMP + INTERVAL '1 year',
    created_by INTEGER REFERENCES api_keys,
    last_used TIMESTAMP WITH TIME ZONE,
    scope INTEGER REFERENCES api_scope,
    deleted BOOL NOT NULL DEFAULT false
);
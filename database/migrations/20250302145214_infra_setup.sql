-- Add migration script here
CREATE TABLE dokument_versions(
    dok_id INTEGER REFERENCES dokument(dok_id) ON DELETE CASCADE,
    previous_id INTEGER REFERENCES dokument(dok_id)  ON DELETE CASCADE,
    PRIMARY KEY(dok_id, previous_id)
);

CREATE TABLE api_scope(
    scope_id INTEGER PRIMARY KEY,
    value VARCHAR NOT NULL UNIQUE
);

INSERT INTO api_scope(scope_id, value) 
VALUES
(1, 'admin'),     -- directly inserting, updating and deleting laws
(2, 'collector'), -- "normal" insertion of laws
(3, 'keyadder');  -- adding new api keys

CREATE TABLE api_keys (
    key_id SERIAL PRIMARY KEY,
    key_hash VARCHAR NOT NULL UNIQUE,
    coll_id UUID UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT NOW() + INTERVAL '1 year',
    created_by INTEGER REFERENCES api_keys(key_id),
    last_used TIMESTAMP WITH TIME ZONE,
    scope_id INTEGER REFERENCES api_scope(scope_id),
    deleted BOOL NOT NULL DEFAULT false
);
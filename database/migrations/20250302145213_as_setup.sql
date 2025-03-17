CREATE TABLE top (
    id SERIAL PRIMARY KEY,
    titel VARCHAR NOT NULL,
    nummer INTEGER NOT NULL
);
CREATE TABLE tops_doks(
    top_id INTEGER NOT NULL REFERENCES top(id) ON DELETE CASCADE,
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    PRIMARY KEY (top_id, dok_id)
);

CREATE TABLE sitzung(
    id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    nummer INTEGER NOT NULL,
    titel VARCHAR,
    link VARCHAR,
    termin TIMESTAMP WITH TIME ZONE NOT NULL,
    public BOOLEAN NOT NULL,
    gr_id INTEGER NOT NULL REFERENCES gremium(id) ON DELETE CASCADE,
    last_update TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
CREATE TABLE rel_s_experten(
    s_id INTEGER NOT NULL REFERENCES sitzung(id) ON DELETE CASCADE,
    exp_id INTEGER NOT NULL REFERENCES autor(id) ON DELETE CASCADE,
    PRIMARY KEY (s_id, exp_id)
);
CREATE TABLE rel_s_tops(
    s_id INTEGER NOT NULL REFERENCES sitzung(id) ON DELETE CASCADE,
    top_id INTEGER NOT NULL REFERENCES top(id) ON DELETE CASCADE,
    PRIMARY KEY (s_id, top_id)
);
CREATE TABLE rel_s_ankuendigung(
    s_id INTEGER NOT NULL REFERENCES sitzung(id) ON DELETE CASCADE,
    dok_id INTEGER NOT NULL REFERENCES dokument(id) ON DELETE CASCADE,
    reihenfolge INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (s_id, dok_id),
    CONSTRAINT unique_order_per_session UNIQUE(s_id, reihenfolge)-- this one enforces a specific order per session
);
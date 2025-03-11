-- Add migration script here
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

CREATE TABLE ausschusssitzung(
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
CREATE TABLE rel_ass_experten(
    ass_id INTEGER NOT NULL REFERENCES ausschusssitzung(id) ON DELETE CASCADE,
    exp_id INTEGER NOT NULL REFERENCES experte(id) ON DELETE CASCADE,
    PRIMARY KEY (ass_id, exp_id)
);
CREATE TABLE rel_ass_tops(
    ass_id INTEGER NOT NULL REFERENCES ausschusssitzung(id) ON DELETE CASCADE,
    top_id INTEGER NOT NULL REFERENCES top(id) ON DELETE CASCADE,
    PRIMARY KEY (ass_id, top_id)
);
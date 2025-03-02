-- Add migration script here
CREATE TABLE top (
    top_id SERIAL PRIMARY KEY,
    titel VARCHAR NOT NULL,
    nummer INTEGER NOT NULL
);
CREATE TABLE tops_doks(
    top_id INTEGER NOT NULL REFERENCES top(top_id) ON DELETE CASCADE,
    dok_id INTEGER NOT NULL REFERENCES dokument(dok_id) ON DELETE CASCADE,
    PRIMARY KEY (top_id, dok_id)
);

CREATE TABLE ausschusssitzung(
    ass_id SERIAL PRIMARY KEY,
    api_id UUID NOT NULL UNIQUE,
    termin TIMESTAMP WITH TIME ZONE NOT NULL,
    public BOOLEAN NOT NULL,
    gr_id INTEGER NOT NULL REFERENCES gremium(gr_id) ON DELETE CASCADE
);
CREATE TABLE rel_ass_experten(
    ass_id INTEGER NOT NULL REFERENCES ausschusssitzung(ass_id) ON DELETE CASCADE,
    exp_id INTEGER NOT NULL REFERENCES experte(exp_id) ON DELETE CASCADE,
    PRIMARY KEY (ass_id, exp_id)
);
CREATE TABLE rel_ass_tops(
    ass_id INTEGER NOT NULL REFERENCES ausschusssitzung(ass_id) ON DELETE CASCADE,
    top_id INTEGER NOT NULL REFERENCES top(top_id) ON DELETE CASCADE,
    PRIMARY KEY (ass_id, top_id)
);
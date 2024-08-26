-- tables that require foreign keys and represent relations
CREATE TABLE
    rel_ges_schlagworte (
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        schlagwort INTEGER REFERENCES schlagworte (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, schlagwort)
    );

CREATE TABLE
    rel_ges_eigenschaft (
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        eigenschaft INTEGER REFERENCES gesetzeseigenschaften (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, eigenschaft)
    );

CREATE TABLE
    rel_ges_status (
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        status INTEGER REFERENCES status (id) ON DELETE CASCADE,
        abstimmung INTEGER REFERENCES abstimmungen (id) ON DELETE CASCADE,
        datum TIMESTAMP NOT NULL, -- the last timestamp is the current status
        PRIMARY KEY (gesetzesvorhaben, status, abstimmung)
    );

CREATE TABLE
    rel_ges_tops (
        top INTEGER REFERENCES tops (id) ON DELETE CASCADE,
        gesetzesvorhaben INTEGER REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        abstimmung INTEGER REFERENCES abstimmungen (id) ON DELETE CASCADE,
        dokument INTEGER REFERENCES dokumente (id) ON DELETE CASCADE,
        titel VARCHAR(255) NOT NULL,
        PRIMARY KEY (top, gesetzesvorhaben, dokument, abstimmung)
    );
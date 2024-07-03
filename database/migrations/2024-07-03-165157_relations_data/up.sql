-- tables that require foreign keys and represent relations
CREATE TABLE
    rel_ges_schlagworte (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        schlagwort SERIAL REFERENCES schlagworte (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, schlagwort)
    );

CREATE TABLE
    rel_ges_abstimmungen (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, abstimmung)
    );

CREATE TABLE
    rel_ges_eigenschaft (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        eigenschaft SERIAL REFERENCES gesetzeseigenschaften (id) ON DELETE CASCADE,
        PRIMARY KEY (gesetzesvorhaben, eigenschaft)
    );

CREATE TABLE
    rel_ges_status (
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        status SERIAL REFERENCES status (id) ON DELETE CASCADE,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        datum DATE NOT NULL,
        active BOOLEAN NOT NULL,
        PRIMARY KEY (gesetzesvorhaben, status, abstimmung)
    );

CREATE TABLE
    rel_ges_tops (
        top SERIAL REFERENCES tops (id),
        gesetzesvorhaben UUID REFERENCES gesetzesvorhaben (id) ON DELETE CASCADE,
        abstimmung SERIAL REFERENCES abstimmungen (id) ON DELETE CASCADE,
        dokument SERIAL REFERENCES dokumente (id) ON DELETE CASCADE,
        titel VARCHAR(255) NOT NULL,
        PRIMARY KEY (top, gesetzesvorhaben, dokument, abstimmung)
    );
// @generated automatically by Diesel CLI.

diesel::table! {
    autor (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        organisation -> Varchar,
    }
}

diesel::table! {
    dokument (id) {
        id -> Int4,
        titel -> Varchar,
        dokumenttyp_id -> Int4,
        url -> Varchar,
        hash -> Varchar,
        zusammenfassung -> Varchar,
    }
}

diesel::table! {
    dokumenttyp (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    gesetzestyp (id) {
        id -> Int4,
        #[max_length = 64]
        value -> Varchar,
    }
}

diesel::table! {
    gesetzesvorhaben (id) {
        id -> Int4,
        api_id -> Uuid,
        titel -> Varchar,
        verfassungsaendernd -> Bool,
        trojaner -> Bool,
        initiative -> Varchar,
        typ -> Int4,
    }
}

diesel::table! {
    identifikatortyp (id) {
        id -> Int4,
        #[max_length = 64]
        value -> Varchar,
    }
}

diesel::table! {
    parlament (id) {
        id -> Int4,
        #[max_length = 2]
        value -> Bpchar,
    }
}

diesel::table! {
    rel_dok_autor (dokument_id, autor_id) {
        dokument_id -> Int4,
        autor_id -> Int4,
    }
}

diesel::table! {
    rel_dok_schlagwort (dokument_id, schlagwort_id) {
        dokument_id -> Int4,
        schlagwort_id -> Int4,
    }
}

diesel::table! {
    rel_gesvh_id (gesetzesvorhaben_id, id_typ, identifikator) {
        gesetzesvorhaben_id -> Int4,
        id_typ -> Int4,
        identifikator -> Varchar,
    }
}

diesel::table! {
    rel_gesvh_links (id) {
        id -> Int4,
        gesetzesvorhaben_id -> Int4,
        link -> Varchar,
    }
}

diesel::table! {
    rel_gesvh_notes (id) {
        id -> Int4,
        gesetzesvorhaben_id -> Int4,
        note -> Varchar,
    }
}

diesel::table! {
    rel_station_dokument (station_id, dokument_id) {
        station_id -> Int4,
        dokument_id -> Int4,
    }
}

diesel::table! {
    rel_station_schlagwort (station_id, schlagwort_id) {
        station_id -> Int4,
        schlagwort_id -> Int4,
    }
}

diesel::table! {
    schlagwort (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    station (id) {
        id -> Int4,
        gesvh_id -> Int4,
        parlament -> Int4,
        stationstyp -> Int4,
        zeitpunkt -> Timestamp,
        url -> Nullable<Varchar>,
        zuordnung -> Varchar,
    }
}

diesel::table! {
    stationstyp (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    stellungnahme (id) {
        id -> Int4,
        titel -> Varchar,
        station_id -> Int4,
        dokument_id -> Int4,
        zeitpunkt -> Timestamp,
        meinung -> Int4,
        url -> Varchar,
    }
}

diesel::joinable!(dokument -> dokumenttyp (dokumenttyp_id));
diesel::joinable!(gesetzesvorhaben -> gesetzestyp (typ));
diesel::joinable!(rel_dok_autor -> autor (autor_id));
diesel::joinable!(rel_dok_autor -> dokument (dokument_id));
diesel::joinable!(rel_dok_schlagwort -> dokument (dokument_id));
diesel::joinable!(rel_dok_schlagwort -> schlagwort (schlagwort_id));
diesel::joinable!(rel_gesvh_id -> gesetzesvorhaben (gesetzesvorhaben_id));
diesel::joinable!(rel_gesvh_id -> identifikatortyp (id_typ));
diesel::joinable!(rel_gesvh_links -> gesetzesvorhaben (gesetzesvorhaben_id));
diesel::joinable!(rel_gesvh_notes -> gesetzesvorhaben (gesetzesvorhaben_id));
diesel::joinable!(rel_station_dokument -> dokument (dokument_id));
diesel::joinable!(rel_station_dokument -> station (station_id));
diesel::joinable!(rel_station_schlagwort -> schlagwort (schlagwort_id));
diesel::joinable!(rel_station_schlagwort -> station (station_id));
diesel::joinable!(station -> gesetzesvorhaben (gesvh_id));
diesel::joinable!(station -> parlament (parlament));
diesel::joinable!(station -> stationstyp (stationstyp));
diesel::joinable!(stellungnahme -> dokument (dokument_id));
diesel::joinable!(stellungnahme -> station (station_id));

diesel::allow_tables_to_appear_in_same_query!(
    autor,
    dokument,
    dokumenttyp,
    gesetzestyp,
    gesetzesvorhaben,
    identifikatortyp,
    parlament,
    rel_dok_autor,
    rel_dok_schlagwort,
    rel_gesvh_id,
    rel_gesvh_links,
    rel_gesvh_notes,
    rel_station_dokument,
    rel_station_schlagwort,
    schlagwort,
    station,
    stationstyp,
    stellungnahme,
);

// @generated automatically by Diesel CLI.

diesel::table! {
    dokument (id) {
        id -> Int4,
        titel -> Varchar,
        zeitpunkt -> Timestamp,
        url -> Varchar,
        hash -> Varchar,
        zusammenfassung -> Nullable<Varchar>,
        dokumententyp -> Int4,
    }
}

diesel::table! {
    dokumententyp (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::table! {
    gesetzestyp (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::table! {
    gesetzesvorhaben (id) {
        id -> Int4,
        api_id -> Uuid,
        titel -> Varchar,
        verfassungsaendernd -> Bool,
        typ -> Int4,
    }
}

diesel::table! {
    identifikatortyp (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::table! {
    parlament (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::table! {
    rel_dok_autor (dokument_id, autor) {
        dokument_id -> Int4,
        autor -> Varchar,
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
    rel_gesvh_init (gesetzesvorhaben, initiator) {
        gesetzesvorhaben -> Int4,
        initiator -> Varchar,
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
        api_key -> Varchar,
    }
}

diesel::table! {
    station (id) {
        id -> Int4,
        gesvh_id -> Int4,
        parlament -> Int4,
        stationstyp -> Int4,
        gremium -> Varchar,
        zeitpunkt -> Timestamp,
        trojaner -> Bool,
        url -> Nullable<Varchar>,
    }
}

diesel::table! {
    stationstyp (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::table! {
    stellungnahme (id) {
        id -> Int4,
        station_id -> Int4,
        dokument_id -> Int4,
        meinung -> Int4,
        lobbyregister -> Nullable<Varchar>,
    }
}

diesel::joinable!(dokument -> dokumententyp (dokumententyp));
diesel::joinable!(gesetzesvorhaben -> gesetzestyp (typ));
diesel::joinable!(rel_dok_autor -> dokument (dokument_id));
diesel::joinable!(rel_dok_schlagwort -> dokument (dokument_id));
diesel::joinable!(rel_dok_schlagwort -> schlagwort (schlagwort_id));
diesel::joinable!(rel_gesvh_id -> gesetzesvorhaben (gesetzesvorhaben_id));
diesel::joinable!(rel_gesvh_id -> identifikatortyp (id_typ));
diesel::joinable!(rel_gesvh_init -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_gesvh_links -> gesetzesvorhaben (gesetzesvorhaben_id));
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
    dokument,
    dokumententyp,
    gesetzestyp,
    gesetzesvorhaben,
    identifikatortyp,
    parlament,
    rel_dok_autor,
    rel_dok_schlagwort,
    rel_gesvh_id,
    rel_gesvh_init,
    rel_gesvh_links,
    rel_station_dokument,
    rel_station_schlagwort,
    schlagwort,
    station,
    stationstyp,
    stellungnahme,
);

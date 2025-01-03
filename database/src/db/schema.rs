// @generated automatically by Diesel CLI.

diesel::table! {
    collector_auth (id) {
        id -> Int4,
        coll_id -> Uuid,
        pubkey -> Varchar,
        deleted -> Bool,
    }
}

diesel::table! {
    dokument (id) {
        id -> Int4,
        titel -> Varchar,
        datum -> Timestamp,
        link -> Varchar,
        hash -> Varchar,
        zusammenfassung -> Nullable<Varchar>,
        typ -> Int4,
    }
}

diesel::table! {
    dokument_versions (dok_id, previous_id) {
        dok_id -> Int4,
        previous_id -> Int4,
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
        verfaend -> Bool,
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
    ip_lastreq (ip) {
        ip -> Int4,
        request_ts -> Timestamp,
    }
}

diesel::table! {
    parlament (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::table! {
    rel_dok_autor (dok_id, autor) {
        dok_id -> Int4,
        autor -> Varchar,
    }
}

diesel::table! {
    rel_dok_schlagwort (dok_id, sw_id) {
        dok_id -> Int4,
        sw_id -> Int4,
    }
}

diesel::table! {
    rel_gsvh_id (gsvh_id, typ, identifikator) {
        gsvh_id -> Int4,
        typ -> Int4,
        identifikator -> Varchar,
    }
}

diesel::table! {
    rel_gsvh_init (gsvh_id, initiator) {
        gsvh_id -> Int4,
        initiator -> Varchar,
    }
}

diesel::table! {
    rel_gsvh_links (id) {
        id -> Int4,
        gsvh_id -> Int4,
        link -> Varchar,
    }
}

diesel::table! {
    rel_station_dokument (stat_id, dok_id) {
        stat_id -> Int4,
        dok_id -> Int4,
    }
}

diesel::table! {
    rel_station_schlagwort (stat_id, sw_id) {
        stat_id -> Int4,
        sw_id -> Int4,
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
        gsvh_id -> Int4,
        parl_id -> Int4,
        typ -> Int4,
        gremium -> Varchar,
        datum -> Timestamp,
        trojaner -> Bool,
        link -> Nullable<Varchar>,
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
        stat_id -> Int4,
        dok_id -> Int4,
        meinung -> Nullable<Int4>,
        lobbyreg_link -> Nullable<Varchar>,
    }
}

diesel::joinable!(dokument -> dokumententyp (typ));
diesel::joinable!(gesetzesvorhaben -> gesetzestyp (typ));
diesel::joinable!(rel_dok_autor -> dokument (dok_id));
diesel::joinable!(rel_dok_schlagwort -> dokument (dok_id));
diesel::joinable!(rel_dok_schlagwort -> schlagwort (sw_id));
diesel::joinable!(rel_gsvh_id -> gesetzesvorhaben (gsvh_id));
diesel::joinable!(rel_gsvh_id -> identifikatortyp (typ));
diesel::joinable!(rel_gsvh_init -> gesetzesvorhaben (gsvh_id));
diesel::joinable!(rel_gsvh_links -> gesetzesvorhaben (gsvh_id));
diesel::joinable!(rel_station_dokument -> dokument (dok_id));
diesel::joinable!(rel_station_dokument -> station (stat_id));
diesel::joinable!(rel_station_schlagwort -> schlagwort (sw_id));
diesel::joinable!(rel_station_schlagwort -> station (stat_id));
diesel::joinable!(station -> gesetzesvorhaben (gsvh_id));
diesel::joinable!(station -> parlament (parl_id));
diesel::joinable!(station -> stationstyp (typ));
diesel::joinable!(stellungnahme -> dokument (dok_id));
diesel::joinable!(stellungnahme -> station (stat_id));

diesel::allow_tables_to_appear_in_same_query!(
    collector_auth,
    dokument,
    dokument_versions,
    dokumententyp,
    gesetzestyp,
    gesetzesvorhaben,
    identifikatortyp,
    ip_lastreq,
    parlament,
    rel_dok_autor,
    rel_dok_schlagwort,
    rel_gsvh_id,
    rel_gsvh_init,
    rel_gsvh_links,
    rel_station_dokument,
    rel_station_schlagwort,
    schlagwort,
    station,
    stationstyp,
    stellungnahme,
);

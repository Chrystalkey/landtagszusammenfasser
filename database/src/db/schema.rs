// @generated automatically by Diesel CLI.

diesel::table! {
    api_keys (id) {
        id -> Int4,
        key_hash -> Varchar,
        coll_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        last_used -> Nullable<Timestamptz>,
        scope -> Nullable<Int4>,
        deleted -> Bool,
    }
}

diesel::table! {
    api_scope (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::table! {
    ausschuss (id) {
        id -> Int4,
        parl_id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    ausschusssitzung (id) {
        id -> Int4,
        termin -> Timestamptz,
        as_id -> Int4,
    }
}

diesel::table! {
    dokument (id) {
        id -> Int4,
        titel -> Varchar,
        last_mod -> Timestamptz,
        volltext -> Nullable<Varchar>,
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
    experte (id) {
        id -> Int4,
        name -> Varchar,
        fachgebiet -> Varchar,
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
    rel_ass_experten (ass_id, exp_id) {
        ass_id -> Int4,
        exp_id -> Int4,
    }
}

diesel::table! {
    rel_ass_tops (ass_id, top_id) {
        ass_id -> Int4,
        top_id -> Int4,
    }
}

diesel::table! {
    rel_dok_autor (dok_id, autor) {
        dok_id -> Int4,
        autor -> Varchar,
    }
}

diesel::table! {
    rel_dok_autorperson (dok_id, autor) {
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
    rel_station_ausschusssitzung (stat_id, as_id) {
        stat_id -> Int4,
        as_id -> Int4,
    }
}

diesel::table! {
    rel_station_dokument (stat_id, dok_id) {
        stat_id -> Int4,
        dok_id -> Int4,
    }
}

diesel::table! {
    rel_station_gesetz (stat_id, gesetz) {
        stat_id -> Int4,
        gesetz -> Varchar,
    }
}

diesel::table! {
    rel_station_schlagwort (stat_id, sw_id) {
        stat_id -> Int4,
        sw_id -> Int4,
    }
}

diesel::table! {
    rel_vorgang_id (vorgang_id, typ, identifikator) {
        vorgang_id -> Int4,
        typ -> Int4,
        identifikator -> Varchar,
    }
}

diesel::table! {
    rel_vorgang_init (vorgang_id, initiator) {
        vorgang_id -> Int4,
        initiator -> Varchar,
    }
}

diesel::table! {
    rel_vorgang_init_person (vorgang_id, initiator) {
        vorgang_id -> Int4,
        initiator -> Varchar,
    }
}

diesel::table! {
    rel_vorgang_links (id) {
        id -> Int4,
        vorgang_id -> Int4,
        link -> Varchar,
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
        vorgang_id -> Int4,
        parl_id -> Int4,
        typ -> Int4,
        titel -> Nullable<Varchar>,
        zeitpunkt -> Nullable<Timestamptz>,
        trojanergefahr -> Nullable<Int4>,
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
        meinung -> Int4,
        lobbyreg_link -> Nullable<Varchar>,
        volltext -> Nullable<Varchar>,
    }
}

diesel::table! {
    top (id) {
        id -> Int4,
        vorgang_id -> Nullable<Int4>,
        titel -> Varchar,
    }
}

diesel::table! {
    vorgang (id) {
        id -> Int4,
        api_id -> Uuid,
        titel -> Varchar,
        verfaend -> Bool,
        wahlperiode -> Int4,
        typ -> Int4,
    }
}

diesel::table! {
    vorgangstyp (id) {
        id -> Int4,
        api_key -> Varchar,
    }
}

diesel::joinable!(api_keys -> api_scope (scope));
diesel::joinable!(ausschuss -> parlament (parl_id));
diesel::joinable!(ausschusssitzung -> ausschuss (as_id));
diesel::joinable!(dokument -> dokumententyp (typ));
diesel::joinable!(rel_ass_experten -> ausschusssitzung (ass_id));
diesel::joinable!(rel_ass_experten -> experte (exp_id));
diesel::joinable!(rel_ass_tops -> ausschusssitzung (ass_id));
diesel::joinable!(rel_ass_tops -> top (top_id));
diesel::joinable!(rel_dok_autor -> dokument (dok_id));
diesel::joinable!(rel_dok_autorperson -> dokument (dok_id));
diesel::joinable!(rel_dok_schlagwort -> dokument (dok_id));
diesel::joinable!(rel_dok_schlagwort -> schlagwort (sw_id));
diesel::joinable!(rel_station_ausschusssitzung -> ausschusssitzung (as_id));
diesel::joinable!(rel_station_ausschusssitzung -> station (stat_id));
diesel::joinable!(rel_station_dokument -> dokument (dok_id));
diesel::joinable!(rel_station_dokument -> station (stat_id));
diesel::joinable!(rel_station_gesetz -> station (stat_id));
diesel::joinable!(rel_station_schlagwort -> schlagwort (sw_id));
diesel::joinable!(rel_station_schlagwort -> station (stat_id));
diesel::joinable!(rel_vorgang_id -> identifikatortyp (typ));
diesel::joinable!(rel_vorgang_id -> vorgang (vorgang_id));
diesel::joinable!(rel_vorgang_init -> vorgang (vorgang_id));
diesel::joinable!(rel_vorgang_init_person -> vorgang (vorgang_id));
diesel::joinable!(rel_vorgang_links -> vorgang (vorgang_id));
diesel::joinable!(station -> parlament (parl_id));
diesel::joinable!(station -> stationstyp (typ));
diesel::joinable!(station -> vorgang (vorgang_id));
diesel::joinable!(stellungnahme -> dokument (dok_id));
diesel::joinable!(stellungnahme -> station (stat_id));
diesel::joinable!(top -> vorgang (vorgang_id));
diesel::joinable!(vorgang -> vorgangstyp (typ));

diesel::allow_tables_to_appear_in_same_query!(
    api_keys,
    api_scope,
    ausschuss,
    ausschusssitzung,
    dokument,
    dokument_versions,
    dokumententyp,
    experte,
    identifikatortyp,
    parlament,
    rel_ass_experten,
    rel_ass_tops,
    rel_dok_autor,
    rel_dok_autorperson,
    rel_dok_schlagwort,
    rel_station_ausschusssitzung,
    rel_station_dokument,
    rel_station_gesetz,
    rel_station_schlagwort,
    rel_vorgang_id,
    rel_vorgang_init,
    rel_vorgang_init_person,
    rel_vorgang_links,
    schlagwort,
    station,
    stationstyp,
    stellungnahme,
    top,
    vorgang,
    vorgangstyp,
);

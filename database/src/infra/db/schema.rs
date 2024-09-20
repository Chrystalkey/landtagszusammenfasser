// @generated automatically by Diesel CLI.

diesel::table! {
    ausschuss (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        parlament -> Int4,
    }
}

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
        api_id -> Uuid,
        #[max_length = 255]
        titel -> Varchar,
        #[max_length = 255]
        identifikator -> Varchar,
        last_access -> Timestamp,
        #[max_length = 512]
        zsmfassung -> Varchar,
        #[max_length = 255]
        url -> Varchar,
        #[max_length = 128]
        hash -> Bpchar,
        doktyp -> Int4,
        gesetzesvorhaben -> Int4,
        station -> Int4,
    }
}

diesel::table! {
    dokumententyp (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    further_links (id) {
        id -> Int4,
        #[max_length = 255]
        link -> Varchar,
        gesetzesvorhaben -> Int4,
    }
}

diesel::table! {
    further_notes (id) {
        id -> Int4,
        #[max_length = 255]
        notes -> Varchar,
        gesetzesvorhaben -> Int4,
    }
}

diesel::table! {
    gesetzestyp (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    gesetzesvorhaben (id) {
        id -> Int4,
        api_id -> Uuid,
        #[max_length = 255]
        titel -> Varchar,
        #[max_length = 128]
        initiator -> Varchar,
        verfassungsaendernd -> Bool,
        trojaner -> Bool,
        typ -> Int4,
        federf -> Nullable<Int4>,
    }
}

diesel::table! {
    parlament (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 2]
        kurzname -> Bpchar,
    }
}

diesel::table! {
    rel_dok_autor (dokument, autor) {
        dokument -> Int4,
        autor -> Int4,
    }
}

diesel::table! {
    rel_station_schlagwort (station, schlagwort) {
        station -> Int4,
        schlagwort -> Int4,
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
        gesetzesvorhaben -> Int4,
        status -> Int4,
        parlament -> Int4,
        #[max_length = 255]
        url -> Nullable<Varchar>,
        api_id -> Uuid,
        datum -> Timestamp,
        ausschuss -> Nullable<Int4>,
        meinungstendenz -> Nullable<Int4>,
    }
}

diesel::table! {
    status (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::joinable!(ausschuss -> parlament (parlament));
diesel::joinable!(dokument -> dokumententyp (doktyp));
diesel::joinable!(dokument -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(dokument -> station (station));
diesel::joinable!(further_links -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(further_notes -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(gesetzesvorhaben -> ausschuss (federf));
diesel::joinable!(gesetzesvorhaben -> gesetzestyp (typ));
diesel::joinable!(rel_dok_autor -> autor (autor));
diesel::joinable!(rel_dok_autor -> dokument (dokument));
diesel::joinable!(rel_station_schlagwort -> schlagwort (schlagwort));
diesel::joinable!(rel_station_schlagwort -> station (station));
diesel::joinable!(station -> ausschuss (ausschuss));
diesel::joinable!(station -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(station -> parlament (parlament));
diesel::joinable!(station -> status (status));

diesel::allow_tables_to_appear_in_same_query!(
    ausschuss,
    autor,
    dokument,
    dokumententyp,
    further_links,
    further_notes,
    gesetzestyp,
    gesetzesvorhaben,
    parlament,
    rel_dok_autor,
    rel_station_schlagwort,
    schlagwort,
    station,
    status,
);

// @generated automatically by Diesel CLI.

diesel::table! {
    abstimmungen (id) {
        id -> Int4,
        ext_id -> Uuid,
        namentlich -> Bool,
        #[max_length = 255]
        url -> Varchar,
        typ -> Nullable<Int4>,
        gesetzesvorhaben -> Nullable<Int4>,
    }
}

diesel::table! {
    abstimmungsergebnisse (id) {
        id -> Int4,
        abstimmung -> Nullable<Int4>,
        fraktion -> Nullable<Int4>,
        anteil -> Float8,
    }
}

diesel::table! {
    abstimmungstyp (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    ausschuesse (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        parlament -> Nullable<Int4>,
    }
}

diesel::table! {
    ausschussberatungen (id) {
        id -> Int4,
        ext_id -> Uuid,
        datum -> Date,
        gesetzesvorhaben -> Nullable<Int4>,
        ausschuss -> Nullable<Int4>,
        dokument -> Nullable<Int4>,
    }
}

diesel::table! {
    dokumente (id) {
        id -> Int4,
        ext_id -> Uuid,
        #[max_length = 255]
        off_id -> Varchar,
        created_at -> Timestamp,
        accessed_at -> Timestamp,
        #[max_length = 255]
        url -> Varchar,
        #[max_length = 255]
        path -> Nullable<Varchar>,
        #[max_length = 64]
        hash -> Bpchar,
        #[max_length = 16]
        filetype -> Varchar,
        gesetzesvorhaben -> Nullable<Int4>,
        doktyp -> Nullable<Int4>,
    }
}

diesel::table! {
    dokumenttypen (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    fraktionen (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    gesetzeseigenschaften (id) {
        id -> Int4,
        #[max_length = 255]
        eigenschaft -> Varchar,
    }
}

diesel::table! {
    gesetzesvorhaben (id) {
        id -> Int4,
        ext_id -> Uuid,
        #[max_length = 255]
        titel -> Varchar,
        #[max_length = 255]
        off_titel -> Varchar,
        #[max_length = 255]
        url_gesblatt -> Nullable<Varchar>,
        #[max_length = 255]
        id_gesblatt -> Nullable<Varchar>,
        verfassungsaendernd -> Bool,
        trojaner -> Bool,
        feder -> Nullable<Int4>,
        initiat -> Nullable<Int4>,
    }
}

diesel::table! {
    initiatoren (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        organisation -> Varchar,
        #[max_length = 255]
        url -> Varchar,
    }
}

diesel::table! {
    parlamente (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 2]
        kurzname -> Bpchar,
    }
}

diesel::table! {
    rel_ges_eigenschaft (gesetzesvorhaben, eigenschaft) {
        gesetzesvorhaben -> Int4,
        eigenschaft -> Int4,
    }
}

diesel::table! {
    rel_ges_schlagworte (gesetzesvorhaben, schlagwort) {
        gesetzesvorhaben -> Int4,
        schlagwort -> Int4,
    }
}

diesel::table! {
    rel_ges_status (gesetzesvorhaben, status) {
        gesetzesvorhaben -> Int4,
        status -> Int4,
        datum -> Timestamp,
    }
}

diesel::table! {
    rel_ges_tops (top, gesetzesvorhaben, dokument, abstimmung) {
        top -> Int4,
        gesetzesvorhaben -> Int4,
        abstimmung -> Int4,
        dokument -> Int4,
        #[max_length = 255]
        titel -> Varchar,
    }
}

diesel::table! {
    schlagworte (id) {
        id -> Int4,
        #[max_length = 255]
        schlagwort -> Varchar,
        #[max_length = 255]
        beschreibung -> Varchar,
    }
}

diesel::table! {
    sonstige_ids (id) {
        id -> Int4,
        gesetzesvorhaben -> Nullable<Int4>,
        #[max_length = 255]
        typ -> Varchar,
        #[max_length = 255]
        inhalt -> Varchar,
    }
}

diesel::table! {
    status (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        parlament -> Nullable<Int4>,
    }
}

diesel::table! {
    tagesordnungspunkt (id) {
        id -> Int4,
        #[max_length = 255]
        titel -> Varchar,
        tops_id -> Nullable<Int4>,
        document -> Nullable<Int4>,
        abstimmung -> Nullable<Int4>,
    }
}

diesel::table! {
    tops (id) {
        id -> Int4,
        ext_id -> Uuid,
        datum -> Date,
        #[max_length = 255]
        url -> Varchar,
        parlament -> Nullable<Int4>,
    }
}

diesel::joinable!(abstimmungen -> abstimmungstyp (typ));
diesel::joinable!(abstimmungen -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(abstimmungsergebnisse -> abstimmungen (abstimmung));
diesel::joinable!(abstimmungsergebnisse -> fraktionen (fraktion));
diesel::joinable!(ausschuesse -> parlamente (parlament));
diesel::joinable!(ausschussberatungen -> ausschuesse (ausschuss));
diesel::joinable!(ausschussberatungen -> dokumente (dokument));
diesel::joinable!(ausschussberatungen -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(dokumente -> dokumenttypen (doktyp));
diesel::joinable!(dokumente -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(gesetzesvorhaben -> ausschuesse (feder));
diesel::joinable!(gesetzesvorhaben -> initiatoren (initiat));
diesel::joinable!(rel_ges_eigenschaft -> gesetzeseigenschaften (eigenschaft));
diesel::joinable!(rel_ges_eigenschaft -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_schlagworte -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_schlagworte -> schlagworte (schlagwort));
diesel::joinable!(rel_ges_status -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_status -> status (status));
diesel::joinable!(rel_ges_tops -> abstimmungen (abstimmung));
diesel::joinable!(rel_ges_tops -> dokumente (dokument));
diesel::joinable!(rel_ges_tops -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_tops -> tops (top));
diesel::joinable!(sonstige_ids -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(status -> parlamente (parlament));
diesel::joinable!(tagesordnungspunkt -> abstimmungen (abstimmung));
diesel::joinable!(tagesordnungspunkt -> dokumente (document));
diesel::joinable!(tagesordnungspunkt -> tops (tops_id));
diesel::joinable!(tops -> parlamente (parlament));

diesel::allow_tables_to_appear_in_same_query!(
    abstimmungen,
    abstimmungsergebnisse,
    abstimmungstyp,
    ausschuesse,
    ausschussberatungen,
    dokumente,
    dokumenttypen,
    fraktionen,
    gesetzeseigenschaften,
    gesetzesvorhaben,
    initiatoren,
    parlamente,
    rel_ges_eigenschaft,
    rel_ges_schlagworte,
    rel_ges_status,
    rel_ges_tops,
    schlagworte,
    sonstige_ids,
    status,
    tagesordnungspunkt,
    tops,
);

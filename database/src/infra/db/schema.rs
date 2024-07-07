// @generated automatically by Diesel CLI.

diesel::table! {
    abstimmungen (id) {
        id -> Int4,
        namentlich -> Bool,
        #[max_length = 255]
        url -> Varchar,
        typ -> Int4,
    }
}

diesel::table! {
    abstimmungsergebnisse (id) {
        id -> Int4,
        abstimmung -> Int4,
        fraktion -> Int4,
        anteil -> Numeric,
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
        parlament -> Int4,
    }
}

diesel::table! {
    ausschussberatungen (id) {
        id -> Int4,
        datum -> Date,
        gesetzesvorhaben -> Nullable<Uuid>,
        ausschuss -> Int4,
        dokument -> Int4,
    }
}

diesel::table! {
    dokumente (id) {
        id -> Int4,
        #[max_length = 255]
        off_id -> Varchar,
        datum -> Date,
        #[max_length = 255]
        url -> Varchar,
        #[max_length = 255]
        collector_url -> Varchar,
        #[max_length = 255]
        file -> Nullable<Varchar>,
        #[max_length = 64]
        hash -> Bpchar,
        gesetzesvorhaben -> Nullable<Uuid>,
        typ -> Int4,
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
        id -> Uuid,
        #[max_length = 255]
        titel -> Varchar,
        #[max_length = 255]
        off_titel -> Varchar,
        #[max_length = 255]
        url_gesblatt -> Nullable<Varchar>,
        #[max_length = 255]
        id_gesblatt -> Nullable<Varchar>,
        verfassungsaendernd -> Bool,
        trojaner -> Nullable<Bool>,
        federfuehrung -> Int4,
        initiator -> Int4,
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
    rel_ges_abstimmungen (gesetzesvorhaben, abstimmung) {
        gesetzesvorhaben -> Uuid,
        abstimmung -> Int4,
    }
}

diesel::table! {
    rel_ges_eigenschaft (gesetzesvorhaben, eigenschaft) {
        gesetzesvorhaben -> Uuid,
        eigenschaft -> Int4,
    }
}

diesel::table! {
    rel_ges_schlagworte (gesetzesvorhaben, schlagwort) {
        gesetzesvorhaben -> Uuid,
        schlagwort -> Int4,
    }
}

diesel::table! {
    rel_ges_status (gesetzesvorhaben, status, abstimmung) {
        gesetzesvorhaben -> Uuid,
        status -> Int4,
        abstimmung -> Int4,
        datum -> Date,
        active -> Bool,
    }
}

diesel::table! {
    rel_ges_tops (top, gesetzesvorhaben, dokument, abstimmung) {
        top -> Int4,
        gesetzesvorhaben -> Uuid,
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
        gesetzesvorhaben -> Nullable<Uuid>,
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
        parlament -> Int4,
    }
}

diesel::table! {
    tops (id) {
        id -> Int4,
        datum -> Date,
        #[max_length = 255]
        url -> Varchar,
        parlament -> Int4,
    }
}

diesel::joinable!(abstimmungen -> abstimmungstyp (typ));
diesel::joinable!(abstimmungsergebnisse -> abstimmungen (abstimmung));
diesel::joinable!(abstimmungsergebnisse -> fraktionen (fraktion));
diesel::joinable!(ausschuesse -> parlamente (parlament));
diesel::joinable!(ausschussberatungen -> ausschuesse (ausschuss));
diesel::joinable!(ausschussberatungen -> dokumente (dokument));
diesel::joinable!(ausschussberatungen -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(dokumente -> dokumenttypen (typ));
diesel::joinable!(dokumente -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(gesetzesvorhaben -> ausschuesse (federfuehrung));
diesel::joinable!(gesetzesvorhaben -> initiatoren (initiator));
diesel::joinable!(rel_ges_abstimmungen -> abstimmungen (abstimmung));
diesel::joinable!(rel_ges_abstimmungen -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_eigenschaft -> gesetzeseigenschaften (eigenschaft));
diesel::joinable!(rel_ges_eigenschaft -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_schlagworte -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_schlagworte -> schlagworte (schlagwort));
diesel::joinable!(rel_ges_status -> abstimmungen (abstimmung));
diesel::joinable!(rel_ges_status -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_status -> status (status));
diesel::joinable!(rel_ges_tops -> abstimmungen (abstimmung));
diesel::joinable!(rel_ges_tops -> dokumente (dokument));
diesel::joinable!(rel_ges_tops -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(rel_ges_tops -> tops (top));
diesel::joinable!(sonstige_ids -> gesetzesvorhaben (gesetzesvorhaben));
diesel::joinable!(status -> parlamente (parlament));
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
    rel_ges_abstimmungen,
    rel_ges_eigenschaft,
    rel_ges_schlagworte,
    rel_ges_status,
    rel_ges_tops,
    schlagworte,
    sonstige_ids,
    status,
    tops,
);

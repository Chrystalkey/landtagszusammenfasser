extern crate diesel_interaction;

use diesel::*;
use diesel_interaction_derive::DieselInteraction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::abstimmungen, belongs_to(Abstimmungstyp, foreign_key=typ) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct Abstimmungen{
    pub id : i32,
    pub ext_id: uuid::Uuid,
    pub namentlich: bool,
    pub url: String,
    pub typ: Option<i32>,
    pub gesetzesvorhaben: Option<i32>,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::abstimmungsergebnisse, belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Fraktionen, foreign_key=fraktion))]
pub struct Abstimmungsergebnisse{
    pub id : i32,
    pub abstimmung: Option<i32>,
    pub fraktion: Option<i32>,
    pub anteil: f64
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::abstimmungstyp)]
pub struct Abstimmungstyp{
    pub id : i32,
    pub name: String,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::ausschuesse, belongs_to(Parlamente, foreign_key=parlament))]
pub struct Ausschuesse{
    pub id : i32,
    pub name: String,
    pub parlament: Option<i32>,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::ausschussberatungen, belongs_to(Ausschuesse, foreign_key=ausschuss) , belongs_to(Dokumente, foreign_key=dokument) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct Ausschussberatungen {
    pub id : i32,
    pub ext_id: uuid::Uuid,
    pub datum: chrono::NaiveDate,
    pub gesetzesvorhaben: Option<i32>,
    pub ausschuss: Option<i32>,
    pub dokument: Option<i32>,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::dokumente, belongs_to(Dokumenttypen, foreign_key=doktyp) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct Dokumente {
    pub id : i32,
    pub ext_id: uuid::Uuid,
    pub off_id: String,
    pub created_at: chrono::NaiveDateTime,
    pub accessed_at: chrono::NaiveDateTime,
    pub url: String,
    pub path: Option<String>,
    pub hash: String,
    pub filetype: String,
    pub gesetzesvorhaben: Option<i32>,
    pub doktyp: Option<i32>,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::dokumenttypen)]
pub struct Dokumenttypen{
    pub id : i32,
    pub name: String,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::fraktionen)]
pub struct Fraktionen{
    pub id : i32,
    pub name: String,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::gesetzeseigenschaften)]
pub struct Gesetzeseigenschaften{
    pub id : i32,
    pub eigenschaft: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::gesetzesvorhaben, belongs_to(Ausschuesse, foreign_key=feder) , belongs_to(Initiatoren, foreign_key=initiat))]
pub struct Gesetzesvorhaben {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub titel: String,
    pub off_titel: String,
    pub url_gesblatt: Option<String>,
    pub id_gesblatt: Option<String>,
    pub verfassungsaendernd: bool,
    pub trojaner: Option<bool>,
    pub feder: Option<i32>,
    pub initiat: Option<i32>,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::initiatoren)]
pub struct Initiatoren {
pub id: i32,
    pub name: String,
    pub organisation: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, Selectable)]
#[diesel(table_name=crate::schema::rel_ges_eigenschaft, primary_key(gesetzesvorhaben,eigenschaft), belongs_to(Gesetzeseigenschaften, foreign_key=eigenschaft) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct RelGesEigenschaft {
    pub gesetzesvorhaben: i32,
    pub eigenschaft: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, Selectable)]
#[diesel(table_name=crate::schema::rel_ges_schlagworte, primary_key(gesetzesvorhaben,schlagwort), belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Schlagworte, foreign_key=schlagwort))]
pub struct RelGesSchlagworte {
    pub gesetzesvorhaben: i32,
    pub schlagwort: i32,
}
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=crate::schema::rel_ges_status, primary_key(gesetzesvorhaben,status,abstimmung), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Status, foreign_key=status))]
pub struct RelGesStatus {
    pub gesetzesvorhaben: i32,
    pub status: i32,
    pub abstimmung: i32,
    pub datum: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=crate::schema::rel_ges_tops, primary_key(top,gesetzesvorhaben,dokument,abstimmung), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Dokumente, foreign_key=dokument) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Tops, foreign_key=top))]
pub struct RelGesTops {
    pub top: i32,
    pub gesetzesvorhaben: i32,
    pub abstimmung: i32,
    pub dokument: i32,
    pub titel: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::schlagworte)]
pub struct Schlagworte {
    pub id: i32,
    pub schlagwort: String,
    pub beschreibung: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::sonstige_ids, belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct SonstigeId {
    pub id: i32,
    pub gesetzesvorhaben: Option<i32>,
    pub typ: String,
    pub inhalt: String,
}
#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::status, belongs_to(Parlamente, foreign_key=parlament))]
pub struct Status {
    pub id: i32,
    pub name: String,
    pub parlament: Option<i32>,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::tagesordnungspunkt, belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Dokumente, foreign_key=document) , belongs_to(Tops, foreign_key=tops_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tagesordnungspunkt {
    pub id : i32,
    pub titel: String,
    pub tops_id: Option<i32>,
    pub document: Option<i32>,
    pub abstimmung: Option<i32>,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::tops, belongs_to(Parlamente, foreign_key=parlament), primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tops {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub datum: chrono::NaiveDate,
    pub url: String,
    pub parlament: Option<i32>,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::parlamente)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Parlamente{
    pub id: i32,
    pub name: String,
    pub kurzname: String,
}
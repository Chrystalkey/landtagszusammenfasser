extern crate diesel_interaction;

use diesel_interaction::*;
use diesel_interaction_derive::DieselInteraction;
use diesel::*;
use super::schema::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type Connection = PgConnection;

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "abstimmungen"]
#[diesel(table_name=abstimmungen, primary_key(id), belongs_to(Abstimmungstyp, foreign_key=typ) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
struct Abstimmungen{
    pub id: i32,
    pub ext_id: Uuid,
    pub namentlich: bool,
    pub url: String,
    pub typ: Option<i32>,
    pub gesetzesvorhaben: Option<i32>,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "abstimmungsergebnisse"]
#[diesel(table_name=abstimmungsergebnisse, primary_key(id), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Fraktionen, foreign_key=fraktion))]
struct Abstimmungsergebnisse{
    pub id: i32,
    pub abstimmung: Option<i32>,
    pub fraktion: Option<i32>,
    pub anteil: f64
}
#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "abstimmungstyp"]
#[diesel(table_name=abstimmungstyp, primary_key(id))]
struct Abstimmungstyp{
    pub id: i32,
    pub name: String,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "ausschuesse"]
#[diesel(table_name=ausschuesse, primary_key(id), belongs_to(Parlamente, foreign_key=parlament))]
struct Ausschuesse{
    pub id: i32,
    pub name: String,
    pub parlament: Option<i32>,
}
#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "ausschussberatungen"]
#[diesel(table_name=ausschussberatungen, primary_key(id), belongs_to(Ausschuesse, foreign_key=ausschuss) , belongs_to(Dokumente, foreign_key=dokument) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct Ausschussberatungen {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub datum: chrono::NaiveDate,
    pub gesetzesvorhaben: Option<i32>,
    pub ausschuss: Option<i32>,
    pub dokument: Option<i32>,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[schema_table = "dokumente"]
#[diesel(table_name=dokumente, primary_key(id), belongs_to(Dokumenttypen, foreign_key=doktyp) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct Dokumente {
    pub id: i32,
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

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "dokumenttypen"]
#[diesel(table_name=dokumenttypen, primary_key(id))]
struct Dokumenttypen{
    pub id: i32,
    pub name: String,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "fraktionen"]
#[diesel(table_name=fraktionen, primary_key(id))]
struct Fraktionen{
    pub id: i32,
    pub name: String,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "gesetzeseigenschaften"]
#[diesel(table_name=gesetzeseigenschaften, primary_key(id))]
struct Gesetzeseigenschaften{
    pub id: i32,
    pub eigenschaft: String,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[schema_table = "gesetzesvorhaben"]
#[diesel(table_name=gesetzesvorhaben, primary_key(id), belongs_to(Ausschuesse, foreign_key=feder) , belongs_to(Initiatoren, foreign_key=initiat))]
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

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "initiatoren"]
#[diesel(table_name=initiatoren, primary_key(id))]
pub struct Initiatoren {
    pub id: i32,
    pub name: String,
    pub organisation: String,
    pub url: String,
}
#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "parlamente"]
#[diesel(table_name=parlamente, primary_key(id))]
pub struct Parlamente {
    pub id: i32,
    pub name: String,
    pub kurzname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_eigenschaft, primary_key(gesetzesvorhaben,eigenschaft), belongs_to(Gesetzeseigenschaften, foreign_key=eigenschaft) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct RelGesEigenschaft {
    pub gesetzesvorhaben: i32,
    pub eigenschaft: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=rel_ges_eigenschaft)]
pub struct UpdateRelGesEigenschaft {
    pub gesetzesvorhaben: Option<i32>,
    pub eigenschaft: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_schlagworte, primary_key(gesetzesvorhaben,schlagwort), belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Schlagworte, foreign_key=schlagwort))]
pub struct RelGesSchlagworte {
    pub gesetzesvorhaben: i32,
    pub schlagwort: i32,
}
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_status, primary_key(gesetzesvorhaben,status,abstimmung), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Status, foreign_key=status))]
pub struct RelGesStatus {
    pub gesetzesvorhaben: i32,
    pub status: i32,
    pub abstimmung: i32,
    pub datum: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_tops, primary_key(top,gesetzesvorhaben,dokument,abstimmung), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Dokumente, foreign_key=dokument) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Top, foreign_key=top))]
pub struct RelGesTop {
    pub top: i32,
    pub gesetzesvorhaben: i32,
    pub abstimmung: i32,
    pub dokument: i32,
    pub titel: String,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "schlagworte"]
#[diesel(table_name=schlagworte, primary_key(id))]
pub struct Schlagworte {
    pub id: i32,
    pub schlagwort: String,
    pub beschreibung: String,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[schema_table = "sonstige_ids"]
#[diesel(table_name=sonstige_ids, primary_key(id), belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct SonstigeId {
    pub id: i32,
    pub gesetzesvorhaben: Option<i32>,
    pub typ: String,
    pub inhalt: String,
}
#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[schema_table = "status"]
#[diesel(table_name=status, primary_key(id), belongs_to(Parlamente, foreign_key=parlament))]
pub struct Status {
    pub id: i32,
    pub name: String,
    pub parlament: Option<i32>,
}

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[schema_table = "tagesordnungspunkt"]
#[diesel(table_name=tagesordnungspunkt, primary_key(id), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Dokumente, foreign_key=document) , belongs_to(Top, foreign_key=tops_id))]
pub struct Tagesordnungspunkt {
    pub id: i32,
    pub titel: String,
    pub tops_id: Option<i32>,
    pub document: Option<i32>,
    pub abstimmung: Option<i32>,
}
#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[schema_table = "tops"]
#[diesel(table_name=tops, primary_key(id), belongs_to(Parlamente, foreign_key=parlament))]
pub struct Top {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub datum: chrono::NaiveDate,
    pub url: String,
    pub parlament: Option<i32>,
}
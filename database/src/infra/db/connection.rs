extern crate diesel_interaction;

use diesel::*;
use diesel_interaction_derive::DieselInteraction;

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::autor)]
pub struct Autor {
    pub id: i32,
    pub name: String,
    pub organisation: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::dokument,
    belongs_to(Station, foreign_key=station), 
    primary_key(id))]
pub struct Dokument {
    pub id : i32,
    pub titel: String,
    pub dokumenttyp_id: i32,
    pub url: String,
    pub hash: String, 
    pub zusammenfassung: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::gesetzesvorhaben)]
pub struct Gesetzesvorhaben {
    pub id: i32, 
    pub api_id: uuid::Uuid,
    pub titel: String,
    pub verfassungsaendernd: bool,
    pub trojaner: bool,
    pub initiative: String,
    pub typ: i32
}
 
#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::station, primary_key(id),
belongs_to(Gesetzesvorhaben, foreign_key=gesvh_id)
)]
pub struct Station{
    pub id: i32,
    pub gesvh_id: i32,
    pub parlament: i32,
    pub stationstyp: i32,
    pub zeitpunkt: chrono::NaiveDateTime,
    pub url: Option<String>,
    pub zuordnung: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::rel_gesvh_links, primary_key(id),
belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct RelGesvhLinks{
    pub id: i32,
    pub gesetzesvorhaben_id: i32, 
    pub link: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::rel_gesvh_notes, primary_key(id), 
belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct RelGesvhNotes{
    pub id: i32,
    pub gesetzesvorhaben_id: i32,
    pub note: String, 
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::stellungnahme, primary_key(id))]
pub struct Stellungnahme{
    pub id: i32,
    pub titel: String,
    pub station_id: i32,
    pub dokument_id: i32,
    pub zeitpunkt: chrono::NaiveDateTime,
    pub meinung: i32,
    pub url: String,
}
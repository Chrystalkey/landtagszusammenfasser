extern crate diesel_interaction;

use diesel::*;
use diesel_interaction_derive::DieselInteraction;

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, QueryableByName, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::ausschuss, 
    belongs_to(Parlament, foreign_key=parlament), 
primary_key(id))]
pub struct Ausschuss{
    pub id : i32,
    pub name: String,
    pub parlament: i32,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::dokumententyp, primary_key(id))]
pub struct Dokumententyp{
    pub id : i32,
    pub value: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::dokument, 
    belongs_to(Dokumententyp, foreign_key=doktyp) , 
    belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben),
    belongs_to(Station, foreign_key=station), primary_key(id))]
pub struct Dokument {
    pub id : i32,
    pub api_id: uuid::Uuid,
    pub titel: String,
    pub identifikator: String, 
    pub last_access: chrono::NaiveDateTime,
    pub zsmfassung: String,
    pub url: String,
    pub hash: String, 
    pub doktyp: i32,
    pub gesetzesvorhaben: i32,
    pub station: i32,
}
#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::station, primary_key(id),
belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben),
belongs_to(Status, foreign_key=status),
belongs_to(Parlament, foreign_key=parlament),
belongs_to(Ausschuss, foreign_key=ausschuss)
)]
pub struct Station{
    pub id: i32, 
    pub gesetzesvorhaben: i32,
    pub status: i32,
    pub parlament: i32, 
    pub api_id: uuid::Uuid,
    pub datum: chrono::NaiveDateTime,
    pub ausschuss: Option<i32>,
    pub meinungstendenz: Option<i32>,
}
#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::gesetzestyp, primary_key(id))]
pub struct Gesetzestyp{
    pub id : i32,
    pub value: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::gesetzesvorhaben, 
    belongs_to(Ausschuss, foreign_key=feder) , 
    belongs_to(Initiatoren, foreign_key=initiat))]
pub struct Gesetzesvorhaben {
    pub id: i32, 
    pub api_id: uuid::Uuid,
    pub titel: String,
    pub initiator: String,
    pub verfassungsaendernd: bool,
    pub trojaner: bool,
    pub typ: i32, 
    pub federf: Option<i32>,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::autor)]
pub struct Autor {
    pub id: i32,
    pub name: String,
    pub organisation: String
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::parlament)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Parlament{
    pub id: i32,
    pub name: String,
    pub kurzname: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name=crate::schema::schlagwort, primary_key(id))]
pub struct Schlagwort {
    pub id: i32,
    pub value: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::status, primary_key(id))]
pub struct Status {
    pub id: i32,
    pub value: String,
}
#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::further_links, primary_key(id),
belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct FurtherLinks{
    pub id: i32,
    pub link: String,
    pub gesetzesvorhaben: i32, 
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, DieselInteraction)]
#[connection_type(deadpool_diesel::postgres::Connection)]
#[diesel(table_name=crate::schema::further_notes, primary_key(id), 
belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct FurtherNotes{
    pub id: i32,
    pub notes: String, 
    pub gesetzesvorhaben: i32,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(table_name=crate::schema::rel_dok_autor, primary_key(dokument, autor),
belongs_to(Dokument, foreign_key=dokument),
belongs_to(Autor, foreign_key=autor))]
pub struct RelDokAutor{
    pub dokument: i32,
    pub autor: i32,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(table_name=crate::schema::rel_station_schlagwort, primary_key(station, schlagwort),
belongs_to(Station, foreign_key=station),
belongs_to(Schlagwort, foreign_key=schlagwort))]
pub struct RelStationSchlagwort{
    pub station: i32,
    pub schlagwort: i32,
}
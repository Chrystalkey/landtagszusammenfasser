use diesel::prelude::*;
use crate::Result;
use crate::db::schema;
use openapi::models::*;

pub async fn gsvh_by_id(id: i32) -> Result<Gesetzesvorhaben> {
    todo!()
}
pub async fn station_by_id(id: i32) -> Result<Station>{
    todo!()
}
pub async fn stellungnahme_by_id(id: i32) -> Result<Stellungnahme>{
    todo!()
}

pub async fn dokument_by_id(id: i32) -> Result<Dokument>{
    todo!()
}
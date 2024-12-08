use std::str::FromStr;

use diesel::prelude::*;
use crate::error::LTZFError;
use crate::Result;
use crate::db::schema;
use openapi::models;
use deadpool_diesel::postgres::Connection;

mod db_models{
    use diesel::prelude::*;
    use uuid::Uuid;

    #[derive(Queryable)]
    pub struct Gesetzesvorhaben {
        pub id: i32,
        pub api_id: Uuid,
        pub verfassungsaendernd: bool,
        pub titel: String,
        pub typ: String
    }
    struct Station{}
    struct Stellungnahme{}
}

pub async fn gsvh_by_id(id: i32, connection: Connection) -> Result<models::Gesetzesvorhaben> {
    use schema::gesetzesvorhaben::dsl;
    let res : db_models::Gesetzesvorhaben
    = connection.interact(move |conn|{
        schema::gesetzesvorhaben::table
        .filter(dsl::id.eq(id))
        .inner_join(schema::gesetzestyp::table)
        .select((dsl::id, dsl::api_id, dsl::verfassungsaendernd, dsl::titel, schema::gesetzestyp::dsl::api_key))
        .get_result(conn)
    }).await??;

    let links = 
    connection.interact(move |conn| schema::rel_gesvh_links::table
    .select(schema::rel_gesvh_links::link)
    .filter(schema::rel_gesvh_links::gesetzesvorhaben_id.eq(res.id))
    .get_results::<String>(conn)).await??;
    let initiatoren = 
    connection.interact(move |conn| schema::rel_gesvh_init::table
    .select(schema::rel_gesvh_init::initiator)
    .filter(schema::rel_gesvh_init::gesetzesvorhaben.eq(res.id))
    .get_results::<String>(conn)).await??;

    let ids = 
    connection.interact(move |conn|{
        schema::rel_gesvh_id::table
        .filter(schema::rel_gesvh_id::gesetzesvorhaben_id.eq(id))
        .inner_join(schema::identifikatortyp::table)
        .select((schema::identifikatortyp::api_key, schema::rel_gesvh_id::identifikator))
        .get_results::<(String, String)>(conn)
    }).await??
    .drain(..)
    .map(|(typ, id)|models::Identifikator{id, typ: models::Identifikatortyp::from_str(&typ).unwrap()})
    .collect();

    let stat_ids: Vec<i32> = 
    connection.interact(move |conn|{
        schema::station::table
        .filter(schema::station::gesvh_id.eq(id))
        .select(schema::station::id)
        .get_results::<i32>(conn)
        }
    ).await??;
    let mut stationen = vec![];
    for sid in stat_ids{
        stationen.push(station_by_id(sid).await?);
    }
    
    Ok(
        models::Gesetzesvorhaben{
            api_id: res.api_id,
            titel: res.titel,
            verfassungsaendernd: res.verfassungsaendernd,
            typ: models::Gesetzestyp::from_str(res.typ.as_str())
            .map_err(|e| LTZFError::GenericStringError(e))?,
            initiatoren,
            ids: Some(ids),
            links: Some(links),
            stationen: stationen,
        }
    )
}
pub async fn station_by_id(id: i32) -> Result<models::Station>{
    todo!()
}
pub async fn stellungnahme_by_id(id: i32) -> Result<models::Stellungnahme>{
    todo!()
}

pub async fn dokument_by_id(id: i32) -> Result<models::Dokument>{
    todo!()
}
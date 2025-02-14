use std::str::FromStr;

use crate::db::schema;
use crate::error::*;
use deadpool_diesel::postgres::Connection;
use diesel::prelude::*;
use openapi::models;

mod db_models {
    use chrono::NaiveDate;
    use diesel::prelude::*;
    use uuid::Uuid;

    #[derive(Queryable)]
    pub struct Gesetzesvorhaben {
        pub id: i32,
        pub api_id: Uuid,
        pub verfassungsaendernd: bool,
        pub titel: String,
        pub typ: String,
    }
    pub struct Station {
        pub parlament: i32,
        pub stationstyp: i32,
        pub gremium: String,
        pub datum: NaiveDate,
        pub trojaner: bool,
        pub link: Option<String>,
    }
}

pub async fn gsvh_by_id(id: i32, connection: &Connection) -> Result<models::Gesetzesvorhaben> {
    use schema::gesetzesvorhaben::dsl;
    let res: db_models::Gesetzesvorhaben = connection
        .interact(move |conn| {
            schema::gesetzesvorhaben::table
                .filter(dsl::id.eq(id))
                .inner_join(schema::gesetzestyp::table)
                .select((
                    dsl::id,
                    dsl::api_id,
                    dsl::verfaend,
                    dsl::titel,
                    schema::gesetzestyp::dsl::api_key,
                ))
                .get_result(conn)
        })
        .await??;

    let links = connection
        .interact(move |conn| {
            schema::rel_gsvh_links::table
                .select(schema::rel_gsvh_links::link)
                .filter(schema::rel_gsvh_links::gsvh_id.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??;
    let initiatoren = connection
        .interact(move |conn| {
            schema::rel_gsvh_init::table
                .select(schema::rel_gsvh_init::initiator)
                .filter(schema::rel_gsvh_init::gsvh_id.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??;
    let init_personen = as_option(connection
        .interact(move |conn| {
            schema::rel_gsvh_init_person::table
                .select(schema::rel_gsvh_init_person::initiator)
                .filter(schema::rel_gsvh_init_person::gsvh_id.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??);

    let ids = connection
        .interact(move |conn| {
            schema::rel_gsvh_id::table
                .filter(schema::rel_gsvh_id::gsvh_id.eq(id))
                .inner_join(schema::identifikatortyp::table)
                .select((
                    schema::identifikatortyp::api_key,
                    schema::rel_gsvh_id::identifikator,
                ))
                .get_results::<(String, String)>(conn)
        })
        .await??
        .drain(..)
        .map(|(typ, id)| models::Identifikator {
            id,
            typ: models::Identifikatortyp::from_str(&typ).unwrap(),
        })
        .collect();

    let stat_ids: Vec<i32> = connection
        .interact(move |conn| {
            schema::station::table
                .filter(schema::station::gsvh_id.eq(id))
                .select(schema::station::id)
                .get_results::<i32>(conn)
        })
        .await??;

    let mut stationen = vec![];
    for sid in stat_ids {
        stationen.push(station_by_id(sid, connection).await?);
    }

    Ok(models::Gesetzesvorhaben {
        api_id: res.api_id,
        titel: res.titel,
        verfassungsaendernd: res.verfassungsaendernd,
        typ: models::Gesetzestyp::from_str(res.typ.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        initiatoren,
        initiator_personen: init_personen,
        ids: Some(ids),
        links: Some(links),
        stationen: stationen,
    })
}

pub async fn station_by_id(id: i32, connection: &Connection) -> Result<models::Station> {
    let mut doks = vec![];
    let dids = connection
        .interact(move |conn| {
            schema::rel_station_dokument::table
                .filter(schema::rel_station_dokument::stat_id.eq(id))
                .select(schema::rel_station_dokument::dok_id)
                .get_results::<i32>(conn)
        })
        .await??;
    for did in dids {
        doks.push(dokument_by_id(did, connection).await?);
    }
    let mut stellungnahmen = vec![];
    let stlid = connection
        .interact(move |conn| {
            schema::stellungnahme::table
                .filter(schema::stellungnahme::stat_id.eq(id))
                .select(schema::stellungnahme::id)
                .get_results::<i32>(conn)
        })
        .await??;

    for sid in stlid {
        stellungnahmen.push(stellungnahme_by_id(sid, connection).await?);
    }

    let schlagworte = connection
        .interact(move |conn| {
            schema::rel_station_schlagwort::table
                .filter(schema::rel_station_schlagwort::stat_id.eq(id))
                .inner_join(schema::schlagwort::table)
                .select(schema::schlagwort::api_key)
                .distinct()
                .get_results::<String>(conn)
        })
        .await??;

    let rval: (
        i32,
        i32,
        String,
        crate::DateTime,
        bool,
        Option<String>,
    ) = connection
        .interact(move |conn| {
            schema::station::table
                .filter(schema::station::id.eq(id))
                .select((
                    schema::station::parl_id,
                    schema::station::typ,
                    schema::station::gremium,
                    schema::station::datum,
                    schema::station::trojaner,
                    schema::station::link,
                ))
                .first(conn)
        })
        .await??;
    let scaffold = db_models::Station {
        parlament: rval.0,
        stationstyp: rval.1,
        gremium: rval.2,
        datum: rval.3.date_naive(),
        trojaner: rval.4,
        link: rval.5,
    };
    let (parl, styp) = connection
        .interact(move |conn| {
            (
                schema::parlament::table
                    .filter(schema::parlament::id.eq(scaffold.parlament))
                    .select(schema::parlament::api_key)
                    .get_result::<String>(conn),
                schema::stationstyp::table
                    .filter(schema::stationstyp::id.eq(scaffold.stationstyp))
                    .select(schema::stationstyp::api_key)
                    .get_result::<String>(conn),
            )
        })
        .await?;
    let betroffene_texte = connection
        .interact(move |conn| {
            schema::rel_station_gesetz::table
                .filter(schema::rel_station_gesetz::stat_id.eq(id))
                .select(schema::rel_station_gesetz::gesetz)
                .get_results::<String>(conn)
        })
        .await??;
    return Ok(models::Station {
        parlament: models::Parlament::from_str(parl?.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        typ: models::Stationstyp::from_str(styp?.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        dokumente: doks,
        schlagworte: Some(schlagworte),
        stellungnahmen: Some(stellungnahmen),
        gremium: scaffold.gremium,
        datum: scaffold.datum,
        betroffene_texte,
        trojaner: Some(scaffold.trojaner),
        link: scaffold.link,
    });
}

pub async fn stellungnahme_by_id(
    id: i32,
    connection: &Connection,
) -> Result<models::Stellungnahme> {
    let rval: (i32, Option<i32>, Option<String>) = connection
        .interact(move |conn| {
            schema::stellungnahme::table
                .filter(schema::stellungnahme::id.eq(id))
                .select((
                    schema::stellungnahme::dok_id,
                    schema::stellungnahme::meinung,
                    schema::stellungnahme::lobbyreg_link,
                ))
                .first(conn)
        })
        .await??;

    return Ok(models::Stellungnahme {
        dokument: dokument_by_id(rval.0, connection).await?,
        meinung: rval.1,
        lobbyregister_link: rval.2,
    });
}

pub async fn dokument_by_id(id: i32, connection: &Connection) -> Result<models::Dokument> {
    let ret: (
        String,
        crate::DateTime,
        String,
        String,
        Option<String>,
        String,
    ) = connection
        .interact(move |conn| {
            use schema::dokument::dsl;
            schema::dokument::table
                .filter(schema::dokument::id.eq(id))
                .inner_join(schema::dokumententyp::table)
                .select((
                    dsl::titel,
                    dsl::datum,
                    dsl::link,
                    dsl::hash,
                    dsl::zusammenfassung,
                    schema::dokumententyp::dsl::api_key,
                ))
                .first(conn)
        })
        .await??;
    let schlagworte: Option<Vec<String>> = as_option(connection
        .interact(move |conn| {
            schema::rel_dok_schlagwort::table
                .filter(schema::rel_dok_schlagwort::dok_id.eq(id))
                .inner_join(schema::schlagwort::table)
                .select(schema::schlagwort::api_key)
                .distinct()
                .get_results(conn)
        })
        .await??);
    let autoren: Option<Vec<String>> = as_option(connection
        .interact(move |conn| {
            schema::rel_dok_autor::table
                .filter(schema::rel_dok_autor::dok_id.eq(id))
                .select(schema::rel_dok_autor::autor)
                .get_results(conn)
        })
        .await??);
    let autorpersonen: Option<Vec<String>> = as_option(connection
        .interact(move |conn| {
            schema::rel_dok_autorperson::table
                .filter(schema::rel_dok_autorperson::dok_id.eq(id))
                .select(schema::rel_dok_autorperson::autor)
                .get_results(conn)
        })
        .await??);

    return Ok(models::Dokument {
        titel: ret.0,
        last_mod: ret.1,
        link: ret.2,
        hash: ret.3,
        zusammenfassung: ret.4,
        schlagworte,
        autoren,
        autorpersonen,
        typ: models::Dokumententyp::from_str(ret.5.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
    });
}

#[derive(QueryableByName, Debug, PartialEq, Eq, Hash, Clone)]
#[diesel(table_name=schema::gesetzesvorhaben)]
struct GSVHID {
    id: i32,
}

pub async fn gsvh_by_parameter(
    params: models::GsvhGetQueryParams,
    connection: &mut Connection,
) -> Result<Vec<models::Gesetzesvorhaben>> {
    let pretable_join = "SELECT gesetzesvorhaben.id, MAX(station.datum) as moddate FROM gesetzesvorhaben, station, gesetzestyp, rel_gsvh_init
    WHERE gesetzesvorhaben.id = station.gsvh_id 
    AND gesetzesvorhaben.id = rel_gsvh_init.gsvh_id
    AND gesetzesvorhaben.typ = gesetzestyp.id";
    let mut param_counter = 0;
    let query = format!(
        "WITH gsvh_moddate as ({}
        {}
        {}
        GROUP BY gesetzesvorhaben.id
    )
    SELECT gsvh_moddate.id FROM gsvh_moddate
    {}
    ORDER BY moddate DESC
    {}
    {}", pretable_join,
        if params.ggtyp.is_some() {param_counter += 1;format!("AND gesetzestyp.api_key = ${}", param_counter)} else {String::new()},
        if params.initiator_contains_any.is_some() {param_counter += 1;format!("AND ${} = ANY(rel_gsvh_init.initiator)", param_counter)} else {String::new()},
        if params.if_modified_since.is_some() {param_counter += 1;format!("AND moddate >= ${}", param_counter)} else {String::new()},
        if params.limit.is_some() {param_counter += 1;format!("LIMIT ${}", param_counter)} else {String::new()},
        if params.offset.is_some() {param_counter += 1;format!("OFFSET ${}", param_counter)} else {String::new()}
    );
    tracing::trace!("Executing Query:\n`{}`", query);
    let mut diesel_query = diesel::sql_query(query)
    .into_boxed();

    if let Some(ggtyp) = params.ggtyp{
        diesel_query = diesel_query.bind::<diesel::sql_types::Text, _>(ggtyp.to_string())
    }
    if let Some(any_init) = params.initiator_contains_any{
        diesel_query = diesel_query.bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(any_init);
    }
    if let Some(moddate) = params.if_modified_since{
        diesel_query = diesel_query.bind::<diesel::sql_types::Timestamp, _>(moddate.naive_utc());
    }
    if let Some(limit) = params.limit{
        diesel_query = diesel_query.bind::<diesel::sql_types::Integer, _>(limit);
    }
    if let Some(offset) = params.offset{
        diesel_query = diesel_query.bind::<diesel::sql_types::Integer, _>(offset);
    }

    let ids: Vec<GSVHID> = connection.interact(
        |conn| diesel_query.load::<GSVHID>(conn)
    ).await??;

    let mut vector = vec![];
    for id in ids{
        vector.push(super::retrieve::gsvh_by_id(id.id, connection).await?);
    }
    Ok(vector)
}

fn as_option<T>(v: Vec<T>) -> Option<Vec<T>> {
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}
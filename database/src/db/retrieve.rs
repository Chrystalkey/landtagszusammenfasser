use std::str::FromStr;

use crate::db::schema;
use crate::error::*;
use deadpool_diesel::postgres::Connection;
use diesel::prelude::*;
use openapi::models;

mod db_models {
    use diesel::prelude::*;
    use uuid::Uuid;

    #[derive(Queryable)]
    pub struct Vorgang {
        pub id: i32,
        pub wahlperiode: i32,
        pub api_id: Uuid,
        pub verfassungsaendernd: bool,
        pub titel: String,
        pub typ: String,
    }
    pub struct Station {
        pub titel: Option<String>,
        pub parlament: i32,
        pub stationstyp: i32,
        pub zeitpunkt: Option<chrono::DateTime<chrono::Utc>>,
        pub trojaner: Option<i32>,
        pub link: Option<String>,
    }
}

pub async fn vorgang_by_id(id: i32, connection: &Connection) -> Result<models::Vorgang> {
    use schema::vorgang::dsl;
    let res: db_models::Vorgang = connection
        .interact(move |conn| {
            schema::vorgang::table
                .filter(dsl::id.eq(id))
                .inner_join(schema::vorgangstyp::table)
                .select((
                    dsl::id,
                    dsl::wahlperiode,
                    dsl::api_id,
                    dsl::verfaend,
                    dsl::titel,
                    schema::vorgangstyp::dsl::api_key,
                ))
                .get_result(conn)
        })
        .await??;

    let links = connection
        .interact(move |conn| {
            schema::rel_vorgang_links::table
                .select(schema::rel_vorgang_links::link)
                .filter(schema::rel_vorgang_links::vorgang_id.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??;
    let initiatoren = connection
        .interact(move |conn| {
            schema::rel_vorgang_init::table
                .select(schema::rel_vorgang_init::initiator)
                .filter(schema::rel_vorgang_init::vorgang_id.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??;
    let init_personen = as_option(connection
        .interact(move |conn| {
            schema::rel_vorgang_init_person::table
                .select(schema::rel_vorgang_init_person::initiator)
                .filter(schema::rel_vorgang_init_person::vorgang_id.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??);

    let ids = connection
        .interact(move |conn| {
            schema::rel_vorgang_id::table
                .filter(schema::rel_vorgang_id::vorgang_id.eq(id))
                .inner_join(schema::identifikatortyp::table)
                .select((
                    schema::identifikatortyp::api_key,
                    schema::rel_vorgang_id::identifikator,
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
                .filter(schema::station::vorgang_id.eq(id))
                .select(schema::station::id)
                .get_results::<i32>(conn)
        })
        .await??;

    let mut stationen = vec![];
    for sid in stat_ids {
        stationen.push(station_by_id(sid, connection).await?);
    }

    Ok(models::Vorgang {
        api_id: res.api_id,
        titel: res.titel,
        wahlperiode: res.wahlperiode as u32,
        verfassungsaendernd: res.verfassungsaendernd,
        typ: models::Vorgangstyp::from_str(res.typ.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        initiatoren,
        initiator_personen: init_personen,
        ids: Some(ids),
        links: Some(links),
        stationen: stationen,
    })
}
pub async fn ausschusssitzung_by_id(id: i32, connection: &Connection) -> Result<models::Ausschusssitzung>{
    todo!("Implement");
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
        Option<crate::DateTime>,
        Option<i32>,
        Option<String>,
        Option<String>,
    ) = connection
        .interact(move |conn| {
            schema::station::table
                .filter(schema::station::id.eq(id))
                .select((
                    schema::station::parl_id,
                    schema::station::typ,
                    schema::station::zeitpunkt,
                    schema::station::trojanergefahr,
                    schema::station::link,
                    schema::station::titel,
                ))
                .first(conn)
        })
        .await??;
    let scaffold = db_models::Station {
        parlament: rval.0,
        stationstyp: rval.1,
        zeitpunkt: rval.2,
        trojaner: rval.3,
        link: rval.4,
        titel: rval.5,
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
    let bt = connection
        .interact(move |conn| {
            schema::rel_station_gesetz::table
                .filter(schema::rel_station_gesetz::stat_id.eq(id))
                .select(schema::rel_station_gesetz::gesetz)
                .get_results::<String>(conn)
        })
        .await??;
    let betroffene_texte = if bt.is_empty(){None}else{Some(bt)};

    let ass_ids : Vec<i32> = connection.interact(move |conn|
        schema::rel_station_ausschusssitzung::table
        .filter(schema::rel_station_ausschusssitzung::stat_id.eq(id))
        .select(schema::rel_station_ausschusssitzung::as_id)
        .get_results::<i32>(conn)
    ).await??;
    let as_objects : Option<Vec<models::Ausschusssitzung>> = if !ass_ids.is_empty() {
        let mut res = vec![];
        for ass_id in ass_ids{
            res.push(ausschusssitzung_by_id(ass_id, connection).await?);
        }
        Some(res)
    } else {
        None
    };

    return Ok(models::Station {
        parlament: models::Parlament::from_str(parl?.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        typ: models::Stationstyp::from_str(styp?.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        dokumente: doks,
        schlagworte: Some(schlagworte),
        stellungnahmen: Some(stellungnahmen),
        zeitpunkt : scaffold.zeitpunkt,
        betroffene_texte,
        trojanergefahr: scaffold.trojaner.map(|x| x as u8),
        titel: scaffold.titel,
        ausschusssitzungen: as_objects,
        link: scaffold.link,
    });
}

pub async fn stellungnahme_by_id(
    id: i32,
    connection: &Connection,
) -> Result<models::Stellungnahme> {
    let rval: (i32, i32, Option<String>, Option<String>) = connection
        .interact(move |conn| {
            schema::stellungnahme::table
                .filter(schema::stellungnahme::id.eq(id))
                .select((
                    schema::stellungnahme::dok_id,
                    schema::stellungnahme::meinung,
                    schema::stellungnahme::lobbyreg_link,
                    schema::stellungnahme::volltext,
                ))
                .first(conn)
        })
        .await??;

    return Ok(models::Stellungnahme {
        dokument: dokument_by_id(rval.0, connection).await?,
        meinung: rval.1 as u8,
        volltext: rval.3,
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
        Option<String>,
    ) = connection
        .interact(move |conn| {
            use schema::dokument::dsl;
            schema::dokument::table
                .filter(schema::dokument::id.eq(id))
                .inner_join(schema::dokumententyp::table)
                .select((
                    dsl::titel,
                    dsl::last_mod,
                    dsl::link,
                    dsl::hash,
                    dsl::zusammenfassung,
                    schema::dokumententyp::dsl::api_key,
                    dsl::volltext,
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
        volltext: ret.6,
        typ: models::Dokumententyp::from_str(ret.5.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
    });
}

#[derive(QueryableByName, Debug, PartialEq, Eq, Hash, Clone)]
#[diesel(table_name=schema::vorgang)]
struct GSVHID {
    id: i32,
}

pub async fn vorgang_by_parameter(
    params: models::VorgangGetQueryParams,
    hparam: models::VorgangGetHeaderParams,
    connection: &mut Connection,
) -> Result<Vec<models::Vorgang>> {
    let pretable_join = "SELECT vorgang.id, MAX(station.datum) as moddate FROM vorgang, station, vorgangstyp, rel_vorgang_init
    WHERE vorgang.id = station.vorgang_id 
    AND vorgang.id = rel_vorgang_init.vorgang_id
    AND vorgang.typ = vorgangstyp.id";
    let mut param_counter = 0;
    let query = format!(
        "WITH vorgang_moddate as ({}
        {}
        {}
        GROUP BY vorgang.id
    )
    SELECT vorgang_moddate.id FROM vorgang_moddate
    {}
    ORDER BY moddate DESC
    {}
    {}", pretable_join,
        if params.ggtyp.is_some() {param_counter += 1;format!("AND vorgangstyp.api_key = ${}", param_counter)} else {String::new()},
        if params.initiator_contains_any.is_some() {param_counter += 1;format!("AND ${} = ANY(rel_gsvh_init.initiator)", param_counter)} else {String::new()},
        if hparam.if_modified_since.is_some() {param_counter += 1;format!("AND moddate >= ${}", param_counter)} else {String::new()},
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
    if let Some(moddate) = hparam.if_modified_since{
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
        vector.push(super::retrieve::vorgang_by_id(id.id, connection).await?);
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
use std::str::FromStr;

use crate::db::schema;
use crate::error::LTZFError;
use crate::Result;
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
        pub zeitpunkt: NaiveDate,
        pub trojaner: bool,
        pub url: Option<String>,
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
                    dsl::verfassungsaendernd,
                    dsl::titel,
                    schema::gesetzestyp::dsl::api_key,
                ))
                .get_result(conn)
        })
        .await??;

    let links = connection
        .interact(move |conn| {
            schema::rel_gesvh_links::table
                .select(schema::rel_gesvh_links::link)
                .filter(schema::rel_gesvh_links::gesetzesvorhaben_id.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??;
    let initiatoren = connection
        .interact(move |conn| {
            schema::rel_gesvh_init::table
                .select(schema::rel_gesvh_init::initiator)
                .filter(schema::rel_gesvh_init::gesetzesvorhaben.eq(res.id))
                .get_results::<String>(conn)
        })
        .await??;

    let ids = connection
        .interact(move |conn| {
            schema::rel_gesvh_id::table
                .filter(schema::rel_gesvh_id::gesetzesvorhaben_id.eq(id))
                .inner_join(schema::identifikatortyp::table)
                .select((
                    schema::identifikatortyp::api_key,
                    schema::rel_gesvh_id::identifikator,
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
                .filter(schema::station::gesvh_id.eq(id))
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
            .map_err(|e| LTZFError::GenericStringError(e))?,
        initiatoren,
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
                .filter(schema::rel_station_dokument::station_id.eq(id))
                .select(schema::rel_station_dokument::dokument_id)
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
                .filter(schema::stellungnahme::station_id.eq(id))
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
                .filter(schema::rel_station_schlagwort::station_id.eq(id))
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
        chrono::NaiveDateTime,
        bool,
        Option<String>,
    ) = connection
        .interact(move |conn| {
            schema::station::table
                .filter(schema::station::id.eq(id))
                .select((
                    schema::station::parlament,
                    schema::station::stationstyp,
                    schema::station::gremium,
                    schema::station::zeitpunkt,
                    schema::station::trojaner,
                    schema::station::url,
                ))
                .first(conn)
        })
        .await??;
    let scaffold = db_models::Station {
        parlament: rval.0,
        stationstyp: rval.1,
        gremium: rval.2,
        zeitpunkt: rval.3.date(),
        trojaner: rval.4,
        url: rval.5,
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

    return Ok(models::Station {
        parlament: models::Parlament::from_str(parl?.as_str())
            .map_err(|e| LTZFError::GenericStringError(e))?,
        typ: models::Stationstyp::from_str(styp?.as_str())
            .map_err(|e| LTZFError::GenericStringError(e))?,
        dokumente: doks,
        schlagworte: Some(schlagworte),
        stellungnahmen: Some(stellungnahmen),
        gremium: scaffold.gremium,
        zeitpunkt: scaffold.zeitpunkt,
        trojaner: Some(scaffold.trojaner),
        url: scaffold.url,
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
                    schema::stellungnahme::dokument_id,
                    schema::stellungnahme::meinung,
                    schema::stellungnahme::lobbyregister,
                ))
                .first(conn)
        })
        .await??;

    return Ok(models::Stellungnahme {
        dokument: dokument_by_id(rval.0, connection).await?,
        meinung: rval.1,
        lobbyregister_url: rval.2,
    });
}

pub async fn dokument_by_id(id: i32, connection: &Connection) -> Result<models::Dokument> {
    let ret: (
        String,
        chrono::NaiveDateTime,
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
                    dsl::zeitpunkt,
                    dsl::url,
                    dsl::hash,
                    dsl::zusammenfassung,
                    schema::dokumententyp::dsl::api_key,
                ))
                .first(conn)
        })
        .await??;
    let sw: Vec<String> = connection
        .interact(move |conn| {
            schema::rel_dok_schlagwort::table
                .filter(schema::rel_dok_schlagwort::dokument_id.eq(id))
                .inner_join(schema::schlagwort::table)
                .select(schema::schlagwort::api_key)
                .distinct()
                .get_results(conn)
        })
        .await??;
    let autoren: Vec<String> = connection
        .interact(move |conn| {
            schema::rel_dok_autor::table
                .filter(schema::rel_dok_autor::dokument_id.eq(id))
                .select(schema::rel_dok_autor::autor)
                .get_results(conn)
        })
        .await??;

    return Ok(models::Dokument {
        titel: ret.0,
        zeitpunkt: ret.1.date(),
        url: ret.2,
        hash: ret.3,
        zusammenfassung: ret.4,
        schlagworte: Some(sw),
        autoren: Some(autoren),
        typ: models::Dokumententyp::from_str(ret.5.as_str())
            .map_err(|e| LTZFError::GenericStringError(e))?,
    });
}

pub async fn gsvh_by_parameter(
    params: models::ApiV1GesetzesvorhabenGetQueryParams,
    connection: &Connection,
) -> Result<Vec<models::Gesetzesvorhaben>> {
    use diesel::prelude::*;

    let mut query = schema::gesetzesvorhaben::table
        .inner_join(schema::station::table.inner_join(schema::parlament::table))
        .into_boxed();
    if let Some(parlament) = params.parlament {
        query = query.filter(schema::parlament::api_key.eq(parlament.to_string()));
    }
    if let Some(x) = params.updated_since {
        query = query.filter(schema::station::zeitpunkt.gt(chrono::NaiveDateTime::from(x)));
    }

    if let Some(x) = params.updated_until {
        query = query.filter(schema::station::zeitpunkt.lt(chrono::NaiveDateTime::from(x)));
    }

    let gsvh_listing: Vec<i32> = connection
        .interact(move |conn| {
            query
                .limit(params.limit.unwrap_or(10) as i64)
                .offset(params.offset.unwrap_or(0) as i64)
                .select(schema::gesetzesvorhaben::id)
                .get_results::<i32>(conn)
        })
        .await??;
    let mut rval = vec![];
    for idx in gsvh_listing {
        rval.push(gsvh_by_id(idx, connection).await?);
    }
    Ok(rval)
}

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
        chrono::NaiveDateTime,
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
        datum: rval.3.date(),
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
                    dsl::datum,
                    dsl::link,
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
                .filter(schema::rel_dok_schlagwort::dok_id.eq(id))
                .inner_join(schema::schlagwort::table)
                .select(schema::schlagwort::api_key)
                .distinct()
                .get_results(conn)
        })
        .await??;
    let autoren: Vec<String> = connection
        .interact(move |conn| {
            schema::rel_dok_autor::table
                .filter(schema::rel_dok_autor::dok_id.eq(id))
                .select(schema::rel_dok_autor::autor)
                .get_results(conn)
        })
        .await??;

    return Ok(models::Dokument {
        titel: ret.0,
        last_mod: ret.1.and_utc(),
        link: ret.2,
        hash: ret.3,
        zusammenfassung: ret.4,
        schlagworte: Some(sw),
        autoren: Some(autoren),
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
    _params: models::ApiV1GesetzesvorhabenGetQueryParams,
    _headers: models::ApiV1GesetzesvorhabenGetHeaderParams,
    _connection: &Connection,
) -> Result<Vec<models::Gesetzesvorhaben>> {
    unimplemented!()
}

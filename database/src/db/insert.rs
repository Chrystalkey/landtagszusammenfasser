use std::collections::HashMap;

use openapi::models;
use crate::Result;
use diesel::prelude::*;
use super::schema;

/// Inserts a new GSVH into the database.
pub fn insert_gsvh(
    api_gsvh: &models::Gesetzesvorhaben,
    connection: &mut diesel::PgConnection
) -> Result<i32> {
    tracing::info!("Inserting complete GSVH into the database");
    use schema::gesetzesvorhaben::dsl;
    use schema::gesetzestyp::dsl as typ_dsl;
    
    let gsvh_id = 
    diesel::insert_into(schema::gesetzesvorhaben::table)
    .values(
        (
            dsl::api_id.eq(api_gsvh.api_id),
            dsl::titel.eq(&api_gsvh.titel),
            dsl::verfassungsaendernd.eq(api_gsvh.verfassungsaendernd),
            dsl::typ.eq(
                typ_dsl::gesetzestyp
                .select(typ_dsl::id)
                .filter(typ_dsl::api_key.eq(&api_gsvh.typ.to_string()))
                .first::<i32>(connection)?
            ),
        )
    )
    .returning(dsl::id)
    .get_result::<i32>(connection)?;

    // insert links, initiatoren, ids
    if let Some(links) = &api_gsvh.links {
        use schema::rel_gsvh_links::dsl as dsl;
        diesel::insert_into(schema::rel_gsvh_links::table)
        .values(
            links.iter()
            .cloned()
            .map(|s|(
                dsl::link.eq(s),
                dsl::gesetzesvorhaben_id.eq(gsvh_id)
            )
            )
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }

    if !api_gsvh.initiatoren.is_empty() {
        use schema::rel_gsvh_init::dsl as dsl;
        diesel::insert_into(schema::rel_gsvh_init::table)
        .values(
            api_gsvh.initiatoren.iter()
            .map(|s|
                (dsl::initiator.eq(s),
                dsl::gesetzesvorhaben.eq(gsvh_id)
            ))
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }

    if let Some(ids) = api_gsvh.ids.as_ref() {
        use schema::rel_gsvh_id::dsl as dsl;
        let mut value_vec = vec![];

        for id_entry in ids.iter(){
            let value= (
                dsl::gesetzesvorhaben_id.eq(gsvh_id),
                dsl::identifikator.eq(&id_entry.id),
                dsl::id_typ.eq(
                    schema::identifikatortyp::table
                    .select(schema::identifikatortyp::id)
                    .filter(schema::identifikatortyp::api_key
                        .eq(&id_entry.typ.to_string())
                    )
                    .first::<i32>(connection)?
                )
            );
            value_vec.push(value);
        }
        diesel::insert_into(schema::rel_gsvh_id::table)
        .values(&value_vec)
        .execute(connection)?;
    }
    
    if !api_gsvh.stationen.is_empty() {
        for stat in api_gsvh.stationen.clone() {
            insert_station(stat, gsvh_id, connection)?;
        }
    }
    tracing::info!("Insertion Successful with ID: {}", gsvh_id);
    Ok(gsvh_id)
}

pub fn insert_station(
    stat: models::Station,
    gsvh_id: i32,
    connection: &mut diesel::PgConnection,
) -> Result<i32> {
    use schema::station::dsl;
    let stat_id = diesel::insert_into(schema::station::table)
    .values(
        (dsl::gsvh_id.eq(gsvh_id),
        dsl::gremium.eq(stat.gremium),
        dsl::trojaner.eq(stat.trojaner.unwrap_or(false)),
        dsl::zeitpunkt.eq(chrono::NaiveDateTime::from(stat.zeitpunkt)),
        dsl::parlament.eq(
            schema::parlament::table.select(schema::parlament::id)
            .filter(schema::parlament::api_key.eq(&stat.parlament.to_string()))
            .first::<i32>(connection)?
        ),
        dsl::stationstyp.eq(
            schema::stationstyp::table.select(schema::stationstyp::id)
            .filter(schema::stationstyp::api_key.eq(&stat.typ.to_string()))
            .first::<i32>(connection)?
        ),
        dsl::url.eq(stat.url),
     )
    )
    .returning(dsl::id)
    .get_result::<i32>(connection)?;
    if !stat.dokumente.is_empty() {
        let mut dok_ids = vec![];
        for dok in stat.dokumente.clone(){
            dok_ids.push(insert_dokument(dok, connection)?);
        }
        
        diesel::insert_into(schema::rel_station_dokument::table)
        .values(
            dok_ids.iter()
            .map(|dok_id|
                (
                    schema::rel_station_dokument::station_id.eq(stat_id),
                    schema::rel_station_dokument::dokument_id.eq(*dok_id)
                )
            )
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }
    if let Some(stln) = stat.stellungnahmen {
        use schema::stellungnahme::dsl;
        for stln in stln {
            diesel::insert_into(schema::stellungnahme::table)
            .values( 
                (
                    dsl::meinung.eq(stln.meinung),
                    dsl::lobbyregister.eq(stln.lobbyregister_url),
                    dsl::station_id.eq(stat_id),
                    dsl::dokument_id.eq(
                        insert_dokument(stln.dokument, connection)?
                    )
                )
            )
            .execute(connection)?;
        }
    }
    if let Some(sw) = stat.schlagworte {
        diesel::insert_into(schema::schlagwort::table)
        .values(
            sw.iter()
            .map(|s|
                schema::schlagwort::api_key.eq(s))
            .collect::<Vec<_>>()
        )
        .on_conflict_do_nothing()
        .execute(connection)?;
        let idvec : HashMap<String, i32> = 
        schema::schlagwort::table
        .filter(schema::schlagwort::api_key.eq_any(&sw))
        .select((schema::schlagwort::api_key, schema::schlagwort::id))
        .get_results::<(String, i32)>(connection)?
        .drain(..).collect();
        tracing::debug!("Inserting Schlagworte: {:?} / {:?}", sw, idvec);

        diesel::insert_into(schema::rel_station_schlagwort::table)
        .values(
            sw.iter()
            .map(|s| {
                (
                    schema::rel_station_schlagwort::station_id.eq(stat_id),
                    schema::rel_station_schlagwort::schlagwort_id.eq(idvec.get(s).unwrap())
                )}
            )
            .collect::<Vec<_>>()
        ).execute(connection)?;
    }

    return Ok(stat_id);
}

pub fn insert_dokument(
    dok: models::Dokument,
    connection: &mut diesel::PgConnection) 
    -> Result<i32> {
    use schema::dokument::dsl;
    let did: i32 = diesel::insert_into(schema::dokument::table)
    .values(
        (
            dsl::titel.eq(dok.titel),
            dsl::url.eq(dok.url),
            dsl::hash.eq(dok.hash),
            dsl::zeitpunkt.eq(chrono::NaiveDateTime::from(dok.zeitpunkt)),
            dsl::zusammenfassung.eq(dok.zusammenfassung),
            dsl::dokumententyp.eq(
                schema::dokumententyp::table.select(schema::dokumententyp::id)
                .filter(schema::dokumententyp::api_key.eq(&dok.typ.to_string()))
                .first::<i32>(connection)?
            )
        )
    )
    .returning(dsl::id)
    .get_result::<i32>(connection)?;
    if let Some(sw) = dok.schlagworte{
        diesel::insert_into(schema::schlagwort::table)
        .values(
            sw.iter()
            .map(|s|
                schema::schlagwort::api_key.eq(s))
            .collect::<Vec<_>>()
        )
        .on_conflict_do_nothing()
        .execute(connection)?;
        let idvec : HashMap<String, i32> = 
        schema::schlagwort::table
        .filter(schema::schlagwort::api_key.eq_any(&sw))
        .select((schema::schlagwort::api_key, schema::schlagwort::id))
        .get_results::<(String, i32)>(connection)?
        .drain(..).collect();
        tracing::debug!("Inserting Schlagworte: {:?} / {:?}", sw, idvec);

        diesel::insert_into(schema::rel_dok_schlagwort::table)
        .values(
            sw.iter()
            .map(|s| {
                (
                    schema::rel_dok_schlagwort::dokument_id.eq(did),
                    schema::rel_dok_schlagwort::schlagwort_id.eq(idvec.get(s).unwrap())
                )}
            )
            .collect::<Vec<_>>()
        ).execute(connection)?;
    }
    if let Some(auth) = dok.autoren{
        diesel::insert_into(schema::rel_dok_autor::table)
        .values(
            auth.iter()
            .map(|s|
                (
                    schema::rel_dok_autor::dokument_id.eq(did),
                    schema::rel_dok_autor::autor.eq(s)
                )
            )
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }
    return Ok(did);
}
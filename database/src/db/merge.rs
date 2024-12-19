/// Handles merging of two datasets.
/// in particular, stellungnahme & dokument are atomic.
/// station and gsvh are not in the sense that gsvh.stations and station.stellungnahmen are appendable and deletable.
/// This means the merge strategy is in general to:
/// 1. find a gsvh that is matching enough
///     a. if found exactly one, update the gsvh, see 2.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new gsvh, return
/// 2. if a., then update the gsvh properties
/// 3. for each station in the new gsvh, find a matching station
///     a. if found exactly one, update it, see 4.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new station & insert
/// 4. if a., then update station properties
/// 5. for each stellungnahme in the new station, find a matching stellungnahme
///    a. if found exactly one, replace it
///    b. if found more than one, send a message to the admins to select one
///    c. if found none, create a new stellungnahme & insert
use super::schema;
use crate::error::*;
use crate::utils;
use crate::{LTZFServer, Result};
use deadpool_diesel::postgres::Connection as AsyncConnection;
use diesel::prelude::*;
use diesel::Connection;
use openapi::models;
use std::collections::HashSet;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    /// this means the api ids are matching, which is a problem.
    ExactlyEqualMatch,
    NoMatch,
}

#[derive(QueryableByName, Debug, PartialEq, Eq, Hash, Clone)]
#[diesel(table_name=schema::gesetzesvorhaben)]
struct GSVHID {
    id: i32,
}
#[derive(QueryableByName, Debug, PartialEq, Eq, Hash, Clone)]
#[diesel(table_name=schema::station)]
struct STATID {
    id: i32,
}

/// this function determines what means "matching enough".
/// I propose:
/// 1. title match: if the titles are similar enough (to be determined)
/// 2. any existing station must match the parliamentary track of the incoming gsvh
///
pub async fn gsvh_merge_candidates(
    model: &models::Gesetzesvorhaben,
    connection: &AsyncConnection,
) -> Result<MergeState<(i32, models::Gesetzesvorhaben)>> {
    let apiid = model.api_id.clone();
    let x = connection
        .interact(move |conn| {
            schema::gesetzesvorhaben::table
                .filter(schema::gesetzesvorhaben::api_id.eq(apiid))
                .select(schema::gesetzesvorhaben::id)
                .first::<i32>(conn)
                .optional()
        })
        .await??;
    if x.is_some() {
        tracing::info!("Found exactly matching GSVH with api_id: {}", model.api_id);
        return Ok(MergeState::ExactlyEqualMatch);
    }

    let result = if let Some(ids) = model.ids.clone() {
        let query = "SELECT id, titel FROM gesetzesvorhaben 
        WHERE NOT EXISTS  (SELECT 1 FROM station, parlament 
            WHERE station.gsvh_id = gesetzesvorhaben.id 
            AND station.parlament = parlament.id 
            AND (parlament.api_key <> $1
                OR parlament.api_key NOT IN ('BT', 'BR') AND $1 NOT IN ('BT', 'BR'))
            )
        AND (SIMILARITY(gesetzesvorhaben.titel, $2) > 0.3 
            OR EXISTS
            (SELECT 1 FROM rel_gsvh_id as rid, identifikatortyp as idt 
                WHERE idt.id = rid.id_typ 
				AND rid.gesetzesvorhaben_id = gesetzesvorhaben.id 
				AND idt.api_key = $3
				AND rid.identifikator = $4)
            )";
        tracing::trace!("Executing Query: {}", query);
        let mut result: HashSet<GSVHID> = HashSet::new();

        for id in ids {
            let stat = model.stationen[0].parlament.to_string();
            let titel = model.titel.clone();
            let id_result = connection
                .interact(move |conn| {
                    diesel::sql_query(query)
                        .bind::<diesel::sql_types::Text, _>(stat)
                        .bind::<diesel::sql_types::Text, _>(titel)
                        .bind::<diesel::sql_types::Text, _>(id.typ.to_string())
                        .bind::<diesel::sql_types::Text, _>(id.id.clone())
                        .get_results::<GSVHID>(conn)
                })
                .await??;

            result.extend(id_result.iter().cloned());
        }
        result.drain().collect::<Vec<_>>()
    } else {
        let query = "SELECT id, titel FROM gesetzesvorhaben 
        WHERE NOT EXISTS  (SELECT 1 FROM station, parlament 
            WHERE station.gsvh_id = gesetzesvorhaben.id 
            AND station.parlament = parlament.id 
            AND (parlament.api_key <> $1
                OR parlament.api_key NOT IN ('BT', 'BR') AND $1 NOT IN ('BT', 'BR')))
        AND SIMILARITY(gesetzesvorhaben.titel, $2) > 0.3";
        tracing::trace!("Executing Query: {}", query);
        let stat = model.stationen[0].parlament.to_string();
        let titel = model.titel.clone();
        let result = connection
            .interact(move |conn| {
                diesel::sql_query(query)
                    .bind::<diesel::sql_types::Text, _>(stat)
                    .bind::<diesel::sql_types::Text, _>(titel)
                    .get_results::<GSVHID>(conn)
            })
            .await??;
        result
    };

    tracing::debug!(
        "Found {} matches for GSVH with api_id: {}\n\n{:?}",
        result.len(),
        model.api_id,
        result
    );

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch((
            result[0].id,
            super::retrieve::gsvh_by_id(result[0].id, connection).await?,
        )),
        _ => {
            let mut asvec = vec![];
            for i in result {
                asvec.push((i.id, super::retrieve::gsvh_by_id(i.id, connection).await?));
            }
            MergeState::AmbiguousMatch(asvec)
        }
    })
}

/// Updates a GSVH based on similarity.
pub fn update_gsvh(
    model: &models::Gesetzesvorhaben,
    candidate: (i32, models::Gesetzesvorhaben),
    connection: &mut PgConnection,
) -> Result<()> {
    let db_id = candidate.0;
    diesel::update(schema::gesetzesvorhaben::table)
        .filter(schema::gesetzesvorhaben::id.eq(db_id))
        .set((
            schema::gesetzesvorhaben::api_id.eq(model.api_id.clone()),
            schema::gesetzesvorhaben::verfassungsaendernd.eq(model.verfassungsaendernd),
        ))
        .execute(connection)?;
    diesel::delete(schema::rel_gsvh_init::table)
        .filter(schema::rel_gsvh_init::gesetzesvorhaben.eq(db_id))
        .execute(connection)?;
    diesel::insert_into(schema::rel_gsvh_init::table)
        .values(
            model
                .initiatoren
                .iter()
                .map(|init| {
                    (
                        schema::rel_gsvh_init::gesetzesvorhaben.eq(db_id),
                        schema::rel_gsvh_init::initiator.eq(init.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .execute(connection)?;
    diesel::delete(schema::rel_gsvh_links::table)
        .filter(schema::rel_gsvh_links::gesetzesvorhaben_id.eq(db_id))
        .execute(connection)?;
    if let Some(links) = model.links.as_ref() {
        diesel::insert_into(schema::rel_gsvh_links::table)
            .values(
                links
                    .iter()
                    .map(|link| {
                        (
                            schema::rel_gsvh_links::gesetzesvorhaben_id.eq(db_id),
                            schema::rel_gsvh_links::link.eq(link.clone()),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(connection)?;
    }

    for station in model.stationen.iter() {
        let similarity_query = format!(
            "SELECT station.id FROM station, stationstyp, parlament
        WHERE
        station.stationstyp = stationstyp.id AND
        station.parlament = parlament.id AND
        stationstyp.api_key = $1 AND
        parlament.api_key = $2 AND
        (SIMILARITY(station.gremium, $3) > 0.5
        OR EXISTS (
            SELECT 1 FROM dokument, rel_station_dokument WHERE 
            rel_station_dokument.station_id = station.id AND
            rel_station_dokument.dokument_id = dokument.id AND
            dokument.hash IN ({})
        )OR EXISTS (
            SELECT 1 FROM dokument, stellungnahme WHERE 
            stellungnahme.station_id = station.id AND
            stellungnahme.dokument_id = dokument.id AND
            dokument.hash IN ({})
        ))",
            station
                .dokumente
                .iter()
                .map(|d| format!("'{}'", d.hash))
                .collect::<Vec<_>>()
                .join(","),
            station
                .stellungnahmen
                .as_ref()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|d| format!("'{}'", d.dokument.hash))
                .collect::<Vec<_>>()
                .join(",")
        );
        let typ = station.typ.clone();
        let parl = station.parlament.clone();
        let gremium = station.gremium.clone();
        let result = diesel::sql_query(similarity_query.as_str())
            .bind::<diesel::sql_types::Text, _>(typ.to_string())
            .bind::<diesel::sql_types::Text, _>(parl.to_string())
            .bind::<diesel::sql_types::Text, _>(gremium)
            .bind::<diesel::sql_types::Integer, _>(db_id)
            .get_results::<STATID>(connection)?
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>();
        if result.is_empty() {
            super::insert::insert_station(station.clone(), db_id, connection)?;
        } else if result.len() == 1 {
            let stat_id = result[0];
            diesel::update(schema::station::table)
                .filter(schema::station::id.eq(stat_id))
                .set((
                    schema::station::gremium.eq(station.gremium.clone()),
                    schema::station::trojaner.eq(station.trojaner.clone().unwrap_or(false)),
                    schema::station::url.eq(station.url.clone()),
                    schema::station::zeitpunkt.eq(chrono::NaiveDateTime::from(station.zeitpunkt)),
                ))
                .execute(connection)?;
            // rep sw
            let schlagworte = station.schlagworte.clone().unwrap_or(vec![]);
            diesel::delete(schema::rel_station_schlagwort::table)
                .filter(schema::rel_station_schlagwort::station_id.eq(stat_id))
                .execute(connection)?;
            diesel::insert_into(schema::schlagwort::table)
                .values(
                    schlagworte
                        .iter()
                        .map(|sw| schema::schlagwort::api_key.eq(sw.clone()))
                        .collect::<Vec<_>>(),
                )
                .on_conflict_do_nothing()
                .execute(connection)?;
            let sw_ids = schema::schlagwort::table
                .select(schema::schlagwort::id)
                .filter(schema::schlagwort::api_key.eq_any(schlagworte))
                .distinct()
                .get_results::<i32>(connection)?;
            diesel::insert_into(schema::rel_station_schlagwort::table)
                .values(
                    sw_ids
                        .iter()
                        .map(|id| {
                            (
                                schema::rel_station_schlagwort::station_id.eq(stat_id),
                                schema::rel_station_schlagwort::schlagwort_id.eq(*id),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .execute(connection)?;
            // rep doks
            for dokument in station.dokumente.iter() {
                let direct_equivalence = schema::dokument::table
                    .select(schema::dokument::id)
                    .filter(schema::dokument::hash.eq(dokument.hash.clone()))
                    .first::<i32>(connection)
                    .optional()?;

                // If the exact same document is already in the database, we can skip this step.
                if direct_equivalence.is_some() {
                    continue;
                }
                let id = super::insert::insert_dokument(dokument.clone(), connection)?;
                diesel::insert_into(schema::rel_station_dokument::table)
                    .values((
                        schema::rel_station_dokument::station_id.eq(stat_id),
                        schema::rel_station_dokument::dokument_id.eq(id),
                    ))
                    .execute(connection)?;
            }
            // rep stln
            if let Some(stellungnahmen) = station.stellungnahmen.clone() {
                for stellungnahme in stellungnahmen.iter() {
                    let direct_equivalence = schema::dokument::table
                        .select(schema::dokument::id)
                        .filter(schema::dokument::hash.eq(stellungnahme.dokument.hash.clone()))
                        .first::<i32>(connection)
                        .optional()?;
                    if direct_equivalence.is_some() {
                        continue;
                    }
                    let dok_id =
                        super::insert::insert_dokument(stellungnahme.dokument.clone(), connection)?;
                    diesel::insert_into(schema::stellungnahme::table)
                        .values((
                            schema::stellungnahme::meinung.eq(stellungnahme.meinung),
                            schema::stellungnahme::dokument_id.eq(dok_id),
                            schema::stellungnahme::station_id.eq(stat_id),
                            schema::stellungnahme::lobbyregister.eq(stellungnahme
                                .lobbyregister_url
                                .clone()
                                .unwrap_or("".to_string())),
                        ))
                        .execute(connection)?;
                }
            }
        } else {
            tracing::warn!("Ambiguous matches for Station");
            todo!("Ambiguous matches for Station, send mail to admins")
        }
    }
    Ok(())
}

pub async fn run(model: &models::Gesetzesvorhaben, server: &LTZFServer) -> Result<()> {
    let connection = server.database.get().await?;
    tracing::debug!(
        "Looking for Merge Candidates for GSVH with api_id: {:?}",
        model.api_id
    );
    let candidates = gsvh_merge_candidates(model, &connection).await?;
    match candidates {
        MergeState::NoMatch => {
            tracing::info!(
                "No Merge Candidate found, Inserting GSVH with api_id: {:?}",
                model.api_id
            );
            let model = model.clone();
            //create GSVH
            connection
                .interact(move |conn| {
                    conn.transaction(|conn| super::insert::insert_gsvh(&model, conn))
                })
                .await??;
        }
        MergeState::OneMatch(one) => {
            tracing::info!(
                "Matching GSVH has api_id: {}, Updating with data from: {}",
                one.1.api_id,
                model.api_id
            );
            let model = model.clone();
            connection
                .interact(move |conn| conn.transaction(move |conn| update_gsvh(&model, one, conn)))
                .await??;
            //update GSVH
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!("Ambiguous matches for GSVH with api_id: {:?}", model.api_id);
            tracing::debug!("Ambiguous matches for GSVH:  {:?} \n\n {:?}", model, many);
            utils::send_email(
                "Ambiguous Match for Merge".to_string(), 
                "Fresh GSVH entered the database, producing ambiguous matches. The new GSVH is: \n\n {:?} \n\n The matches are: \n\n {:?}\n please provide guidance.".to_string(),
            server)?;
            return Err(DataValidationError::AmbiguousMatch {
                message: format!(
                    "Merge Candidates: {:?}",
                    many.iter().map(|e| e.1.api_id).collect::<Vec<_>>()
                ),
            }.into());
        }
        MergeState::ExactlyEqualMatch => {
            return Err(DataValidationError::DuplicateApiId{id: model.api_id}.into());
        }
    }
    Ok(())
}


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
use crate::{LTZFServer, Result};
use crate::error::LTZFError;
use deadpool_diesel::postgres::Connection as AsyncConnection;
use diesel::Connection;
use diesel::{prelude::*};
use openapi::models;
use crate::utils;
use super::schema;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    /// this means the api ids are matching, which is a problem.
    ExactlyEqualMatch,
    NoMatch,
}

#[derive(QueryableByName, Debug)]
#[diesel(table_name=schema::gesetzesvorhaben)]
struct GSVHID{
    id: i32
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
    let x = connection.interact(move |conn|
        schema::gesetzesvorhaben::table
        .filter(schema::gesetzesvorhaben::api_id.eq(apiid))
        .select(schema::gesetzesvorhaben::id)
        .first::<i32>(conn)
        .optional()
    ).await??;
    if x.is_some(){
        tracing::info!("Found exactly matching GSVH with api_id: {}", model.api_id);
        return Ok(MergeState::ExactlyEqualMatch);
    }

    let query = "SELECT id, titel FROM gesetzesvorhaben 
    WHERE SIMILARITY(titel, '$1') > 0.3
    AND NOT EXISTS 
    (SELECT * FROM station, parlament WHERE station.gsvh_id = gesetzesvorhaben.id AND station.parlament = parlament.id AND parlament.api_key <> '$2')";

    tracing::debug!("Executing Query: {}", query);
    let dquery = 
    diesel::sql_query(query)
    .bind::<diesel::sql_types::Text, _>(model.titel.clone())
    .bind::<diesel::sql_types::Text, _>(model.stationen[0].parlament.to_string());
    let result =
        connection.interact(move |conn|
            dquery.get_results::<GSVHID>(conn)
        ).await??;
    tracing::debug!("Found {} matches for GSVH with api_id: {}\n\n{:?}", result.len(), model.api_id, result);
    
    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch((result[0].id, super::retrieve::gsvh_by_id(result[0].id, connection).await?)),
        _ => {
            let mut asvec = vec![];
            for i in result{
                asvec.push((i.id, super::retrieve::gsvh_by_id(i.id, connection).await?));
            }
            MergeState::AmbiguousMatch(asvec)
        },
    })
}

pub async fn update_gsvh(
    model: &models::Gesetzesvorhaben,
    candidate: (i32, models::Gesetzesvorhaben),
    connection: &AsyncConnection,
) -> Result<()> {
    Ok(todo!("update_gsvh"))
}

pub async fn run(model: &models::Gesetzesvorhaben, server: &LTZFServer) -> Result<()> {
    let connection = server.database.get().await?;
    tracing::debug!("Looking for Merge Candidates for GSVH with api_id: {:?}", model.api_id);
    let candidates = gsvh_merge_candidates(model, &connection).await?;
    match candidates {
        MergeState::NoMatch => {
            tracing::info!("No Merge Candidate found, Inserting GSVH with api_id: {:?}", model.api_id);
            let model = model.clone();
            //create GSVH
            connection.interact(
                move |conn| 
                    conn.transaction(
                        |conn| super::insert::insert_gsvh(&model, conn)
                    )
            ).await??;
        }
        MergeState::OneMatch(one) => {
            tracing::info!("Matching GSVH has api_id: {}, Updating with data from: {}", one.1.api_id, model.api_id);
            update_gsvh(model, one, &connection).await?;
            //update GSVH
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!("Ambiguous matches for GSVH with api_id: {:?}", model.api_id);
            tracing::debug!("Ambiguous matches for GSVH:  {:?} \n\n {:?}", model, many);
            utils::send_email(
                "Ambiguous Match for Merge".to_string(), 
                "Fresh GSVH entered the database, producing ambiguous matches. The new GSVH is: \n\n {:?} \n\n The matches are: \n\n {:?}\n please provide guidance.".to_string(),
            server)?;
        }
        MergeState::ExactlyEqualMatch => {
            return Err(LTZFError::ApiIDEqual(model.api_id));
        }
    }
    Ok(())
}

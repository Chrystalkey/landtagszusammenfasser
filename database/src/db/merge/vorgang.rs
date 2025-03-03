#![allow(unused)]
/// Handles merging of two datasets.
/// in particular, stellungnahme & dokument are atomic.
/// station and vorgang are not in the sense that vorgang.stations and station.stellungnahmen are appendable and deletable.
/// This means the merge strategy is in general to:
/// 1. find a vorgang that is matching enough
///     a. if found exactly one, update the vorgang, see 2.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new vorgang, return
/// 2. if a., then update the vorgang properties
/// 3. for each station in the new vorgang, find a matching station
///     a. if found exactly one, update it, see 4.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new station & insert
/// 4. if a., then update station properties
/// 5. for each stellungnahme in the new station, find a matching stellungnahme
///    a. if found exactly one, replace it
///    b. if found more than one, send a message to the admins to select one
///    c. if found none, create a new stellungnahme & insert

use crate::{LTZFServer, Result};
use crate::db::insert;
use openapi::models;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    NoMatch,
}

/// this function determines what means "matching enough".
/// 1. wenn api_id matcht
/// 2. wenn wp, typ und mindestens ein identifikator matchen
/// [TODO]3. wenn wp, typ und 1/5 des volltextes sich "sehr ähnlich sind (tbd)"
pub async fn vorgang_merge_candidates(
    model: &models::Vorgang,
    executor: impl sqlx::PgExecutor<'_>,
    srv: &LTZFServer,
) -> Result<MergeState<i32>> {
    let obj = "merged Vorgang";
    let ident_t: Vec<_> = model.ids.as_ref().unwrap_or(&vec![]).iter().map(|x|x.id.clone()).collect();
    let identt_t: Vec<_> = model.ids.as_ref().unwrap_or(&vec![]).iter().map(|x| srv.guard_ts(x.typ, model.api_id, obj).unwrap()).collect();
    let result = sqlx::query!(
        "WITH db_id_table AS (
            SELECT rel_vg_ident.vg_id as vg_id, identifikator as ident, vg_ident_typ.value as idt_str
            FROM vg_ident_typ, rel_vg_ident 
            WHERE vg_ident_typ.id = rel_vg_ident.typ)

        SELECT DISTINCT(vorgang.id), vorgang.api_id FROM vorgang
        INNER JOIN vorgangstyp vt ON vt.id = vorgang.typ
        WHERE
        vorgang.api_id = $1 OR
        (
        vorgang.wahlperiode = $4 AND 
        vt.value = $5 AND
            EXISTS (SELECT * FROM UNNEST($2::text[], $3::text[]) as eingabe(ident, typ), db_id_table WHERE 
                db_id_table.vg_id = vorgang.id AND
                eingabe.ident = db_id_table.ident AND
                eingabe.typ = db_id_table.idt_str
    ));", 
    model.api_id, &ident_t[..], &identt_t[..], model.wahlperiode as i32, 
    srv.guard_ts(model.typ, model.api_id, obj)?)
    .fetch_all(executor).await?;

    tracing::debug!("Found {} matches for Vorgang with api_id: {}",result.len(),model.api_id);

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch(result[0].id),
        _ => {
            tracing::warn!("Warning: Mehrere Vorgänge gefunden, die als Kandidaten für Merge infrage kommen für den Vorgang `{}`:\n{:?}", 
            model.api_id, result.iter().map(|r|r.api_id).collect::<Vec<_>>());
            MergeState::AmbiguousMatch(
                result.iter().map(|x|x.id).collect()
            )
        }
    })
}

/// bei gleichem Vorgang => Vorraussetzung
/// 1. wenn die api_id matcht
/// 2. wenn typ, parlament matcht und mindestens ein Dokument gleich ist
pub async fn station_merge_candidates(model: &models::Station, vorgang: i32, executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer)-> Result<MergeState<i32>> {
    let obj = "merged station";
    todo!()
}
pub async fn dokument_merge_candidates(model: &models::Dokument, executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer)->Result<MergeState<i32>>{
    todo!()
}

pub async fn execute_merge_dokument (
    model: &models::Dokument,
    candidate: i32,
    executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    todo!()
}
pub async fn execute_merge_station (
    model: &models::Station,
    candidate: i32,
    executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    todo!()
}
pub async fn execute_merge_vorgang (
    model: &models::Vorgang,
    candidate: i32,
    executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    todo!()
}

pub async fn run_integration(model: &models::Vorgang, server: &LTZFServer) -> Result<()> {
    let mut tx = server.sqlx_db.begin().await?;
    tracing::debug!(
        "Looking for Merge Candidates for Vorgang with api_id: {:?}",
        model.api_id);
    let candidates = vorgang_merge_candidates(model, &mut *tx, server).await?;
    match candidates {
        MergeState::NoMatch => {
            tracing::info!(
                "No Merge Candidate found, Inserting Complete Vorgang with api_id: {:?}",
                model.api_id
            );
            let model = model.clone();
            insert::insert_vorgang(&model, &mut tx, server).await?;
        }
        MergeState::OneMatch(one) => {
            let api_id = sqlx::query!("SELECT api_id FROM vorgang WHERE id = $1", one)
            .map(|r|r.api_id).fetch_one(&mut *tx).await?;
            tracing::info!(
                "Matching Vorgang in the DB has api_id: {}, Updating with data from: {}",
                api_id,
                model.api_id
            );
            let model = model.clone();
            execute_merge_vorgang(&model, one, &mut *tx, server).await?;
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!("Ambiguous matches for Vorgang with api_id: {:?}", model.api_id);
            tracing::debug!("Details:  {:?} \n\n {:?}", model, many);
            unimplemented!("Notify Admins via $WAY");
        }
    }
    tx.commit().await?;
    Ok(())
}

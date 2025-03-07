use openapi::models;
use crate::{LTZFServer, Result};
use crate::db::insert;
use super::MergeState;

/// bei entweder
/// - gleicher api id oder
/// - gleichem gremium und Termin
pub async fn ass_merge_candidates(
    model: &models::Ausschusssitzung,
    tx: &mut sqlx::PgTransaction<'_>
) -> Result<MergeState<i32>> {
    let cands = sqlx::query!("
        SELECT a.id FROM ausschusssitzung a
        INNER JOIN gremium g on g.id = a.gr_id
        INNER JOIN parlament p ON g.parl = p.id
        WHERE a.api_id = $1 OR
        (p.value = $2 AND g.name = $3 AND g.wp = $4 AND a.termin = $5)
        ", model.api_id, model.ausschuss.parlament.to_string(), 
        model.ausschuss.name, model.ausschuss.wahlperiode as i32,
        model.termin).map(|r|r.id).fetch_all(&mut **tx).await?;
    Ok(match cands.len(){
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch(cands[0]),
        _ => MergeState::AmbiguousMatch(cands)
    })
}

/// here is a special case: tops are just replaced, not solved with UNION.
/// except when the input is an empty vec, then the previous db state is kept as-is
/// replacable data points are: 
/// - termin
/// - public
pub async fn execute_merge_ass(
    model: &models::Ausschusssitzung,
    candidate: i32,
    tx: &mut sqlx::PgTransaction<'_>,
    srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    sqlx::query!("UPDATE ausschusssitzung SET 
    termin = $1, public = $2 WHERE id = $3", 
    model.termin, model.public, db_id)
    .execute(&mut **tx).await?;
    if model.tops.is_empty(){
        return Ok(())
    }
    sqlx::query!("DELETE FROM top 
    WHERE EXISTS (
        SELECT 1 FROM rel_ass_tops rat 
        WHERE top.id = rat.top_id AND rat.ass_id = $1)
    ", db_id)
    .execute(&mut **tx).await?;
    let mut tids = vec![];
    for top in &model.tops{
        tids.push(insert::insert_top(top, tx, &srv).await?);
    }
    sqlx::query!("INSERT INTO rel_ass_tops(top_id, ass_id)
    SELECT blub, $1 FROM UNNEST($2::int4[]) blub", db_id, &tids[..])
    .execute(&mut **tx).await?;
    Ok(())
}

pub async fn run_integration(model: &models::Ausschusssitzung, server: &LTZFServer) -> Result<()> {
    let mut tx = server.sqlx_db.begin().await?;
    tracing::debug!(
        "Looking for Merge Candidates for Ausschusssitzung with api_id: {:?}",
        model.api_id);
    let candidates = ass_merge_candidates(model, &mut tx).await?;
    match candidates {
        MergeState::NoMatch => {
            tracing::info!(
                "No Merge Candidate found, Inserting Complete Ausschusssitzung with api_id: {:?}",
                model.api_id
            );
            let model = model.clone();
            insert::insert_ausschusssitzung(&model, &mut tx, server).await?;
        }
        MergeState::OneMatch(one) => {
            let api_id = sqlx::query!("SELECT api_id FROM ausschusssitzung WHERE id = $1", one)
            .map(|r|r.api_id).fetch_one(&mut *tx).await?;
            tracing::info!(
                "Matching Ausschusssitzung in the DB has api_id: {}, Updating with data from: {:?}",
                api_id,
                model.api_id
            );
            let model = model.clone();
            execute_merge_ass(&model, one, &mut tx, server).await?;
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!("Ambiguous matches for Ausschusssitzung with api_id: {:?}", model.api_id);
            tracing::debug!("Details:  {:?} \n\n {:?}", model, many);
            unimplemented!("Notify Admins via $WAY");
        }
    }
    tx.commit().await?;
    Ok(())
}
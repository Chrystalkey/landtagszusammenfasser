use crate::async_db;
use crate::infra::api::IdentifikatorTyp;
use diesel::prelude::*;
use std::sync::Arc;

extern crate diesel_interaction;
use crate::error::{DatabaseError, LTZFError};
use crate::infra::api;
use crate::infra::db::connection as dbcon;
use crate::infra::db::schema as dbschema;
use crate::AppState;
use axum::http::StatusCode;

/// A gesvh is mergeable if:
/// new.titel == old.titel || new.ids(vorgang_id) == old.ids(vorgang_id)
async fn merge_candidates(
    gesvh: &api::Gesetzesvorhaben,
    conn: &mut deadpool_diesel::postgres::Connection,
) -> Result<Vec<i32>, DatabaseError> {
    let titel = gesvh.titel.clone();
    let mut eligible_gesvh: Vec<i32> = conn
        .interact(move |conn: &mut PgConnection| {
            dbschema::gesetzesvorhaben::table
                .filter(dbschema::gesetzesvorhaben::titel.eq(titel))
                .select(dbschema::gesetzesvorhaben::id)
                .load::<i32>(conn)
        })
        .await
        .map_err(diesel_interaction::DieselInteractionError::from)
        .map_err(DatabaseError::from)?
        .map_err(DatabaseError::from)?;

    let id_elements: Vec<api::Identifikator> = gesvh
        .ids
        .iter()
        .filter(|&el| el.typ == IdentifikatorTyp::Vorgangsnummer)
        .cloned()
        .collect();

    if id_elements.len() > 0 {
        let result: Vec<i32> = conn
            .interact(move |conn| {
                dbschema::gesetzesvorhaben::table
                    .inner_join(
                        dbschema::rel_gesvh_id::table.inner_join(dbschema::identifikatortyp::table),
                    )
                    .filter(
                        dbschema::rel_gesvh_id::identifikator
                            .eq(&id_elements[0].id)
                            .and(dbschema::identifikatortyp::value.eq("Vorgangsnummer")),
                    )
                    .select(dbschema::gesetzesvorhaben::id)
                    .load::<i32>(conn)
            })
            .await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
            .map_err(DatabaseError::from)?;
        eligible_gesvh.extend(result);
    }
    Ok(eligible_gesvh)
}

fn merge_gesvh(
    gesvh: api::Gesetzesvorhaben,
    conn: &mut diesel::pg::PgConnection,
) -> Result<(), DatabaseError> {
    todo!()
}
fn create_gesvh(
    gesvh: api::Gesetzesvorhaben,
    conn: &mut diesel::pg::PgConnection,
) -> Result<(), DatabaseError> {
    let typ_variant = gesvh.typ;
    let gesvh_object = dbcon::gesetzesvorhaben::Insert{
        api_id: gesvh.api_id, 
        initiative: gesvh.initiative,
        titel: gesvh.titel,
        trojaner: gesvh.trojaner, 
        verfassungsaendernd: gesvh.verfassungsaendernd, 
        typ: dbschema::gesetzestyp::table
                .filter(dbschema::gesetzestyp::value.eq(
                    format!("{:?}", typ_variant))
                )
                .select(dbschema::gesetzestyp::id)
                .first::<i32>(conn)?,
    };
    let gesvh_id = diesel::insert_into(dbschema::gesetzesvorhaben::table)
            .values(&gesvh_object)
            .returning(dbschema::gesetzesvorhaben::id)
            .get_result::<i32>(conn)?;
    // TODO: You stopped here
    // insert the gesvh itself
    // insert stations
    // Insert links & notes
    Ok(())
}

/// Used to create gesetzesvorhaben & associated data with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    object: api::Gesetzesvorhaben,
) -> std::result::Result<StatusCode, LTZFError> {
    let mut conn = app.pool.get().await.map_err(DatabaseError::from)?;

    let merge_candidates = merge_candidates(&object, &mut conn).await?;
    tracing::info!("Mergeable: {}", merge_candidates.len() == 1);
    if merge_candidates.len() > 1 {
        tracing::warn!("Error: Newly Posted Gesetzesvorhaben has more than one 
        candidate for a merge. It will be inserted as a new entry, please review manually.\n
        Candidate IDs: {:?}\nGesetzesvorhaben: {:?}", &merge_candidates, &object);
        // TODO: send a mail message
        return Err(DatabaseError::MultipleMergeCandidates(merge_candidates, object).into());
    }
    conn.interact(move |conn| {
        conn.transaction(|conn| {
            if merge_candidates.len() == 1 {
                merge_gesvh(object, conn)
            } else {
                create_gesvh(object, conn)
            }
        })
    })
    .await
    .map_err(DatabaseError::from)??;
    return Ok(StatusCode::CREATED);
}

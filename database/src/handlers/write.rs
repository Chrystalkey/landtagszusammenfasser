use std::sync::Arc;

extern crate diesel_interaction;
use crate::external::no_match_found;
use crate::infra::db::connection as dbcon;
use crate::infra::api;
use crate::AppState;
use crate::error::*;
use axum::http::StatusCode;
use diesel::Connection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, OptionalExtension};
use uuid::Uuid;

fn create_gesvh(
    gesvh: api::Gesetzesvorhaben,
    app: Arc<AppState>,
    conn: &mut diesel::pg::PgConnection,
) -> ::std::result::Result<(), DatabaseError> {
    use crate::schema::gesetzesvorhaben as gm;

    let gen_id = Uuid::now_v7();
    tracing::trace!("Generating Id: {}", gen_id);
    Ok(())
}

fn create_stationen(gesvh_id: i32, stationen: Vec<FatOption<api::Station, i32>>, conn: &mut diesel::pg::PgConnection, app: Arc<AppState>) -> std::result::Result<(), DatabaseError>{
    
    Ok(())
}

/// Used to create gesetzesvorhaben & associated data with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    cupdate: api::CUPUpdate,
) -> std::result::Result<StatusCode, LTZFError> {
    let gesvh = cupdate.payload;
    let conn = app.pool.get()
    .await.map_err(DatabaseError::from)?;
    tracing::debug!("Starting Insertion Transaction");
    todo!();
    tracing::debug!("Finished Insertion Transaction");
    tracing::info!("Inserted New Gesetzesvorhaben into Database");
    return Ok(StatusCode::CREATED);
}

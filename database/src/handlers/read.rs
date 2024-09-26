use crate::async_db;

use crate::infra::api::Response;
use crate::AppState;
use crate::error::{Result, DatabaseError};
use crate::infra::db::connection as dbcon;
use crate::infra::api;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) async fn get_gesvh(app: Arc<AppState>, gesvh_id: Uuid) -> Result<api::Response> {
    let mut conn = app.pool.get().await.map_err(DatabaseError::from)?;
    let result : dbcon::Gesetzesvorhaben = async_db!(
        conn, first,
        {
            dbcon::gesetzesvorhaben::table
                .filter(dbcon::gesetzesvorhaben::module::api_id.eq(gesvh_id))
        }
    );
    return Ok(
        api::Response{
            payload: vec![api::Gesetzesvorhaben::construct_from(result, &mut conn).await?]
        }
    );
}
pub(crate) async fn get_gesvh_filtered(
    app: Arc<AppState>,
    filters: crate::router::filters::GesVHFilter,
) -> Result<api::Response> {
    let conn = app.pool.get()
    .await.map_err(DatabaseError::from)?;
    todo!();
}
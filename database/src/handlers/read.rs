use crate::async_db;

use crate::AppState;
use crate::error::{Result, DatabaseError};
use crate::infra::db::connection as dbcon;
use crate::infra::api;
use std::sync::Arc;
use diesel::prelude::*;
use uuid::Uuid;

pub(crate) async fn get_gesvh(app: Arc<AppState>, gesvh_id: Uuid) -> Result<api::WSResponse> {
    let conn = app.pool.get().await.map_err(DatabaseError::from)?;
    let result : dbcon::Gesetzesvorhaben = async_db!(
        conn, first,
        {
            dbcon::gesetzesvorhaben::table
                .filter(dbcon::gesetzesvorhaben::module::api_id.eq(gesvh_id))
        }
    );
    return Ok(
        api::WSResponse{
            id: Uuid::now_v7(),
            payload: api::WSPayload::Gesetzesvorhaben(
                vec![api::Gesetzesvorhaben::construct_from(result, conn).await?]
            ),
        }
    );
}
pub(crate) async fn get_gesvh_filtered(
    app: Arc<AppState>,
    filters: crate::router::filters::GesVHFilter,
) -> Result<api::WSResponse> {
    let conn = app.pool.get()
    .await.map_err(DatabaseError::from)?;
    let mut result : Vec<dbcon::Gesetzesvorhaben> = async_db!(
        conn, load,
        {
            let mut query = dbcon::gesetzesvorhaben::table
            .inner_join(crate::schema::station::table.inner_join(crate::schema::parlament::table))
            .select(dbcon::gesetzesvorhaben::table::all_columns())
            .into_boxed();
            if let Some(since) = filters.updated_since{
                query = query.filter(dbcon::station::module::datum.ge(since.naive_utc()));
            }
            if let Some(until) = filters.updated_until{
                query = query.filter(dbcon::station::module::datum.le(until.naive_utc()));
            }
            if let Some(limit) = filters.limit{
                query = query.limit(limit as i64);
            }
            if let Some(offset) = filters.offset{
                query = query.offset(offset as i64);
            }
            if let Some(parlament) = filters.parlament{
                query = 
                    query.filter(dbcon::parlament::module::kurzname.eq(
                        parlament.to_uppercase()
                    ));
            }
            query
        }
    );
    let mut api_vec = vec![];
    for gesvh in result.drain(..){
        api_vec.push(
            api::Gesetzesvorhaben::construct_from(gesvh, 
                app.pool.get().await.map_err(DatabaseError::from)?
            ).await?
        );
    }

    return Ok(
        api::WSResponse{
            id: Uuid::now_v7(),
            payload: api::WSPayload::Gesetzesvorhaben(api_vec),
        }
    );
}
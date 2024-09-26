use std::sync::Arc;

extern crate diesel_interaction;
use crate::infra::db::connection as dbcon;
use crate::infra::api;
use crate::AppState;
use crate::error::{DatabaseError, LTZFError};
use axum::http::StatusCode;

use crate::infra::db::schema as dbschema;
use diesel::prelude::*;

/// A gesvh is mergeable if: 
/// new.titel == old.titel || new.off_titel == old.new_titel ||
/// new.ids(vorgang_id) == old.ids(vorgang_id)
async fn is_mergable(gesvh: &api::Gesetzesvorhaben, conn: &mut deadpool_diesel::postgres::Connection) 
-> Result<bool, DatabaseError> {todo!()}

fn merge_gesvh(gesvh: api::Gesetzesvorhaben, conn: &mut diesel::pg::PgConnection) 
-> Result<(), DatabaseError> {todo!()}
fn create_gesvh(gesvh: api::Gesetzesvorhaben, conn: &mut diesel::pg::PgConnection) 
-> Result<(), DatabaseError> {todo!()}

/// Used to create gesetzesvorhaben & associated data with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    object: api::Gesetzesvorhaben,
) -> std::result::Result<StatusCode, LTZFError> {
    let mut conn = 
    app.pool.get().await.map_err(DatabaseError::from)?;

    let is_mergeable = is_mergable(&object, &mut conn).await?;
    conn.interact(
        move |conn| {
            conn.transaction(
                |conn|{
                    if is_mergeable{
                        merge_gesvh(object, conn)
                    }else{
                        create_gesvh(object, conn)
                    }
                }
            )
        }
    ).await
    .map_err(DatabaseError::from)??;
    return Ok(StatusCode::CREATED);
}

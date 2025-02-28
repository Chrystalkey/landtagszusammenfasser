use crate::{LTZFServer, Result};
use crate::db::schema;
use uuid::Uuid;
use diesel::prelude::*;
use openapi::apis::default::VorgangDeleteResponse;

pub async fn delete_vorgang_by_api_id(
    api_id: Uuid,
    server: &LTZFServer
) -> Result<VorgangDeleteResponse>{
    let connection = server.database.get().await?;
    let aff_rows  = connection.interact(move |conn|{
        diesel::delete(schema::vorgang::table)
        .filter(schema::vorgang::api_id.eq(api_id))
        .execute(conn)
    }).await??;
    if aff_rows == 0{
        return Ok(VorgangDeleteResponse::Status404_NoElementWithThisID);
    }
    Ok(VorgangDeleteResponse::Status204_DeletedSuccessfully)
}
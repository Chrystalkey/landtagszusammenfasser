use crate::{LTZFServer, Result};
use diesel::prelude::*;
use openapi::models;

pub async fn api_v1_vorgang_id_get(
    server: &LTZFServer,
    path_params: models::VorgangGetByIdPathParams,
) -> Result<models::Vorgang> {
    tracing::info!("api_v1_vorgang_gesvh_id_get called");
    use crate::db::schema;
    let connection = server.database.get().await?;
    let gsid = connection
        .interact(move |conn| {
            schema::vorgang::table
                .filter(schema::vorgang::api_id.eq(path_params.vorgang_id))
                .select(schema::vorgang::id)
                .first::<i32>(conn)
        })
        .await??;

    Ok(crate::db::retrieve::vorgang_by_id(gsid, &connection).await?)
}

pub async fn api_v1_vorgang_get(
    server: &LTZFServer,
    query_params: models::VorgangGetQueryParams,
    header_params: models::VorgangGetHeaderParams,
) -> Result<models::VorgangGet200Response> {
    tracing::info!("api_v1_vorgang_get called");
    let mut connection = server.database.get().await?;
    let vorgang = crate::db::retrieve::vorgang_by_parameter(query_params,header_params, &mut connection).await?;
    Ok(models::VorgangGet200Response {
        payload: if vorgang.is_empty() { None } else { Some(vorgang) },
    })
}

use crate::{LTZFServer, Result};
use crate::utils::as_option;
use openapi::models;

pub async fn api_v1_vorgang_id_get(
    server: &LTZFServer,
    path_params: models::VorgangGetByIdPathParams,
) -> Result<models::Vorgang> {
    tracing::info!("api_v1_vorgang_id_get called");
    let mut tx = server.sqlx_db.begin().await?;
    let dbid = sqlx::query!("SELECT id FROM vorgang WHERE api_id = $1", path_params.vorgang_id)
    .map(|x|x.id).fetch_one(&mut *tx).await?;
    let result = crate::db::retrieve::vorgang_by_id(dbid, &mut tx).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn api_v1_vorgang_get(
    server: &LTZFServer,
    query_params: models::VorgangGetQueryParams,
    header_params: models::VorgangGetHeaderParams,
) -> Result<models::VorgangGet200Response> {
    tracing::info!("api_v1_vorgang_get called");
    let mut tx = server.sqlx_db.begin().await?;
    let result = crate::db::retrieve::vorgang_by_parameter(query_params,header_params, &mut tx).await?;
    tx.commit().await?;
    Ok(models::VorgangGet200Response {payload: as_option(result)})
}

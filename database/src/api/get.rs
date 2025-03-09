use crate::{db::retrieve, LTZFServer, Result};
use crate::utils::as_option;
use openapi::models;

pub async fn vg_id_get(
    server: &LTZFServer,
    path_params: models::VorgangGetByIdPathParams,
) -> Result<models::Vorgang> {
    let mut tx = server.sqlx_db.begin().await?;
    let dbid = sqlx::query!("SELECT id FROM vorgang WHERE api_id = $1", path_params.vorgang_id)
    .map(|x|x.id).fetch_one(&mut *tx).await?;
    let result = retrieve::vorgang_by_id(dbid, &mut tx).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn vg_get(
    server: &LTZFServer,
    query_params: models::VorgangGetQueryParams,
) -> Result<models::VorgangGet200Response> {
    let mut tx = server.sqlx_db.begin().await?;
    let result = retrieve::vorgang_by_parameter(query_params, &mut tx).await?;
    tx.commit().await?;
    Ok(models::VorgangGet200Response {payload: as_option(result)})
}

pub async fn as_get_by_id(
    server: &LTZFServer,
    path_params: models::AsGetByIdPathParams,
) -> Result<openapi::apis::default::AsGetByIdResponse> {
    use openapi::apis::default::AsGetByIdResponse;
    let mut tx = server.sqlx_db.begin().await?;
    let api_id = path_params.as_id;
    let id = sqlx::query!("SELECT id FROM ausschusssitzung WHERE api_id = $1",api_id)
    .map(|r|r.id).fetch_one(&mut *tx).await?;
    let result = retrieve::ausschusssitzung_by_id(id,&mut tx).await?;
    tx.commit().await?;
    Ok(AsGetByIdResponse::Status200_SuccessfulOperation(result))
}

pub async fn as_get(
    server: &LTZFServer,
    query_params: models::AsGetQueryParams,
) -> Result<models::AsGet200Response> {
    let mut tx = server.sqlx_db.begin().await?;
    let result = retrieve::as_by_parameter(query_params, &mut tx).await?;
    tx.commit().await?;
    Ok(models::AsGet200Response {payload: as_option(result)})
}

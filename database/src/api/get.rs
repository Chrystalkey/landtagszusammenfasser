use crate::utils::as_option;
use crate::{db::retrieve, LTZFServer, Result};
use openapi::models;

pub async fn vg_id_get(
    server: &LTZFServer,
    header_params: &models::VorgangGetByIdHeaderParams,
    path_params: &models::VorgangGetByIdPathParams,
) -> Result<models::Vorgang> {
    let mut tx = server.sqlx_db.begin().await?;
    let dbid = sqlx::query!(
        "SELECT id FROM vorgang WHERE api_id = $1 AND EXISTS (
            SELECT 1 FROM station s WHERE s.zp_modifiziert > COALESCE($2, CAST('1940-01-01T00:00:00Z' AS TIMESTAMPTZ))
        )",
        path_params.vorgang_id,
        header_params.if_modified_since
    )
    .map(|x| x.id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(dbid) = dbid{
        let result = retrieve::vorgang_by_id(dbid, &mut tx).await?;
        tx.commit().await?;
        Ok(result)
    } else {
        Err(crate::error::LTZFError::Validation { source:  crate::error::DataValidationError::QueryParametersNotSatisfied})
    }
}

pub async fn vg_get(
    server: &LTZFServer,
    header_params: &models::VorgangGetHeaderParams,
    query_params: &models::VorgangGetQueryParams,
) -> Result<models::VorgangGet200Response> {
    let mut tx = server.sqlx_db.begin().await?;
    let result = retrieve::vorgang_by_parameter(query_params, header_params, &mut tx).await?;
    tx.commit().await?;
    Ok(models::VorgangGet200Response {
        payload: as_option(result),
    })
}

pub async fn s_get_by_id(
    server: &LTZFServer,
    header_params: &models::SGetByIdHeaderParams,
    path_params: &models::SGetByIdPathParams,
) -> Result<openapi::apis::default::SGetByIdResponse> {
    use openapi::apis::default::SGetByIdResponse;
    let mut tx = server.sqlx_db.begin().await?;
    let api_id = path_params.sid;
    let id = sqlx::query!("
    SELECT id FROM sitzung WHERE api_id = $1
    AND last_update > COALESCE($2, CAST('1940-01-01T00:00:00' AS TIMESTAMPTZ));", 
    api_id, header_params.if_modified_since)
        .map(|r| r.id)
        .fetch_optional(&mut *tx)
        .await?;
    if let Some(id) = id{
        let result = retrieve::sitzung_by_id(id, &mut tx).await?;
        tx.commit().await?;
        Ok(SGetByIdResponse::Status200_SuccessfulOperation(result))
    }else{
        Err(crate::error::LTZFError::Validation { source: crate::error::DataValidationError::QueryParametersNotSatisfied })
    }
}

pub async fn s_get(
    server: &LTZFServer,
    header_params: &models::SGetHeaderParams,
    query_params: &models::SGetQueryParams,
) -> Result<models::SGet200Response> {
    let mut tx = server.sqlx_db.begin().await?;
    let result = retrieve::sitzung_by_param(query_params, header_params,  &mut tx).await?;
    tx.commit().await?;
    Ok(models::SGet200Response {
        payload: as_option(result),
    })
}

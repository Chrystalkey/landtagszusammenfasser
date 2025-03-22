use crate::db::{delete, insert, merge, retrieve};
use crate::{LTZFServer, Result};
use openapi::{
    apis::default::{SidPutResponse, VorgangIdPutResponse},
    models,
};
use crate::utils::as_option;

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
) -> Result<Vec<models::Vorgang>> {
    let mut tx = server.sqlx_db.begin().await?;
    let result = retrieve::vorgang_by_parameter(query_params, header_params, &mut tx).await?;
    tx.commit().await?;
    Ok(result)
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
) -> Result<Vec<models::Sitzung>> {
    let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = server.sqlx_db.begin().await?;
    let result = retrieve::sitzung_by_param(query_params, header_params,  &mut tx).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn vorgang_id_put(
    server: &LTZFServer,
    path_params: &models::VorgangIdPutPathParams,
    body: &models::Vorgang,
) -> Result<VorgangIdPutResponse> {
    let mut tx = server.sqlx_db.begin().await?;
    let api_id = path_params.vorgang_id;
    let db_id = sqlx::query!("SELECT id FROM vorgang WHERE api_id = $1", api_id)
        .map(|x| x.id)
        .fetch_one(&mut *tx)
        .await?;
    let db_cmpvg = retrieve::vorgang_by_id(db_id, &mut tx).await?;
    if db_cmpvg == *body {
        return Ok(VorgangIdPutResponse::Status204_ContentUnchanged);
    }
    match delete::delete_vorgang_by_api_id(api_id, server).await? {
        openapi::apis::default::VorgangDeleteResponse::Status204_DeletedSuccessfully => {
            insert::insert_vorgang(&body, &mut tx, server).await?;
        }
        _ => {
            unreachable!("If this is reached, some assumptions did not hold")
        }
    }

    tx.commit().await?;
    Ok(VorgangIdPutResponse::Status201_Created)
}

pub async fn vorgang_put(server: &LTZFServer, model: &models::Vorgang) -> Result<()> {
    tracing::trace!("api_v1_vorgang_put called");
    merge::vorgang::run_integration(&model, server).await?;
    Ok(())
}

pub async fn s_id_put(
    server: &LTZFServer,
    path_params: &models::SidPutPathParams,
    body: &models::Sitzung,
) -> Result<SidPutResponse> {
    use openapi::apis::default::*;
    let mut tx = server.sqlx_db.begin().await?;
    let api_id = path_params.sid;
    let db_id = sqlx::query!("SELECT id FROM sitzung WHERE api_id = $1", api_id)
        .map(|x| x.id)
        .fetch_one(&mut *tx)
        .await?;
    let db_cmpvg = retrieve::sitzung_by_id(db_id, &mut tx).await?;
    if db_cmpvg == *body {
        return Ok(SidPutResponse::Status204_NotModified);
    }
    match delete::delete_ass_by_api_id(api_id, server).await? {
        SitzungDeleteResponse::Status204_DeletedSuccessfully => {
            insert::insert_sitzung(&body, &mut tx, server).await?;
        }
        _ => {
            unreachable!("If this is reached, some assumptions did not hold")
        }
    }

    tx.commit().await?;
    Ok(SidPutResponse::Status201_Created)
}

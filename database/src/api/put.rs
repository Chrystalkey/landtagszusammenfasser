use crate::db::{delete, insert, merge, retrieve};
use crate::{LTZFServer, Result};
use openapi::{
    apis::default::{SidPutResponse, VorgangIdPutResponse},
    models,
};

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

use crate::{Result, LTZFServer};
use openapi::{apis::default::VorgangIdPutResponse, models};
use crate::db::{delete, insert, retrieve, merge};

pub async fn api_v1_vorgang_id_put(
    server: &LTZFServer,
    path_params: models::VorgangIdPutPathParams,
    body: models::Vorgang,
) -> Result<VorgangIdPutResponse> {
    tracing::trace!("api_v1_vorgang_id_put called");
    let mut tx = server.sqlx_db.begin().await?;
    let api_id = path_params.vorgang_id;
    let db_id = sqlx::query!("SELECT id FROM vorgang WHERE api_id = $1", api_id)
    .map(|x|x.id).fetch_one(&mut *tx).await?;
    let db_cmpvg = retrieve::vorgang_by_id(db_id, &mut tx).await?;
    if db_cmpvg == body {
        return Ok(VorgangIdPutResponse::Status204_ContentUnchanged);
    }
    match delete::delete_vorgang_by_api_id(api_id, server).await?{
        openapi::apis::default::VorgangDeleteResponse::Status204_DeletedSuccessfully => {
            insert::insert_vorgang(&body, &mut tx, server).await?;
        },
        _ => {unreachable!("If this is reached, some assumptions did not hold")}
    }

    tx.commit().await?;
    Ok(VorgangIdPutResponse::Status201_Created)
}

pub async fn api_v1_vorgang_put(
    server: &LTZFServer,
    api_vorgang: models::Vorgang
)-> Result<()> {
    tracing::trace!("api_v1_vorgang_put called");
    merge::vorgang::run_integration(&api_vorgang, server).await?;
    Ok(())
}

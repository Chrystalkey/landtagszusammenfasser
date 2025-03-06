use crate::{LTZFServer, Result};
use uuid::Uuid;
use openapi::apis::default::{VorgangDeleteResponse, AsDeleteResponse};

pub async fn delete_vorgang_by_api_id(
    api_id: Uuid,
    server: &LTZFServer
) -> Result<VorgangDeleteResponse> {
    let thing = sqlx::query!("DELETE FROM vorgang WHERE api_id = $1 RETURNING id", api_id)
    .fetch_optional(&server.sqlx_db).await?;

    if thing.is_none() {
        return Ok(VorgangDeleteResponse::Status404_NoElementWithThisID);
    }
    Ok(VorgangDeleteResponse::Status204_DeletedSuccessfully)
}
pub async fn delete_ass_by_api_id(
    api_id: Uuid,
    server: &LTZFServer
) -> Result<AsDeleteResponse> {
    let thing = sqlx::query!("DELETE FROM ausschusssitzung WHERE api_id = $1 RETURNING id", api_id)
    .fetch_optional(&server.sqlx_db).await?;

    if thing.is_none() {
        return Ok(AsDeleteResponse::Status404_NoElementWithThisID);
    }
    Ok(AsDeleteResponse::Status204_DeletedSuccessfully)
}
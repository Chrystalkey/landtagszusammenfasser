use crate::{Result, LTZFServer};
use openapi::{apis::default::VorgangIdPutResponse, models};

pub async fn api_v1_vorgang_id_put(
    _server: &LTZFServer,
    _path_params: models::VorgangIdPutPathParams,
    _body: models::Vorgang,
) -> Result<VorgangIdPutResponse>{
    todo!();
}
use crate::{Result, LTZFServer};
use openapi::{apis::default::VorgangPutResponse, models};

pub async fn api_v1_vorgang_id_put(
    _server: &LTZFServer,
    _path_params: models::VorgangPutPathParams,
    _body: models::Vorgang,
) -> Result<VorgangPutResponse>{
    todo!();
}
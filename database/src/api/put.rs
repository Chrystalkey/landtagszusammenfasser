use crate::{Result, LTZFServer};
use openapi::{apis::default::GsvhPutResponse, models};

pub async fn api_v1_gesetzesvorhaben_gsvh_id_put(
    _server: &LTZFServer,
    _path_params: models::GsvhPutPathParams,
    _body: models::Gesetzesvorhaben,
) -> Result<GsvhPutResponse>{
    todo!();
}
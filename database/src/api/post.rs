use crate::{Result, LTZFServer, db};
use openapi::models;

pub async fn api_v1_gesetzesvorhaben_post(
    server: &LTZFServer,
    api_gsvh: models::Gesetzesvorhaben
)-> Result<()> {
    db::merge::run(&api_gsvh, server).await?;
    Ok(())
}

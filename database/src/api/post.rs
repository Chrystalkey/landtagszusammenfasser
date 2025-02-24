use crate::{Result, LTZFServer, db};
use openapi::models;

pub async fn api_v1_vorgang_post(
    server: &LTZFServer,
    api_gsvh: models::Vorgang
)-> Result<()> {
    tracing::trace!("api_v1_vorgang_post called");
    db::merge::run(&api_gsvh, server).await?;
    Ok(())
}

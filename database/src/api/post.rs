use crate::{Result, LTZFServer, db};
use openapi::models;

pub async fn api_v1_vorgang_post(
    server: &LTZFServer,
    api_vorgang: models::Vorgang
)-> Result<()> {
    tracing::trace!("api_v1_vorgang_post called");
    db::vgmerge::run_integration(&api_vorgang, server).await?;
    Ok(())
}

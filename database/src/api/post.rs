use crate::{Result, LTZFServer, db};
use diesel::Connection;
use openapi::models;

pub async fn api_v1_gesetzesvorhaben_post(
    server: &LTZFServer,
    api_gsvh: models::Gesetzesvorhaben
)-> Result<()> {
    let conn = server.database.get().await?;
    let res = conn.interact(
        move |conn| 
            conn.transaction(
                |conn| db::insert::insert_gsvh(&api_gsvh, conn)
            )
    ).await??;
    Ok(())
}

use crate::{LTZFServer, Result};
use diesel::prelude::*;
use openapi::models;

pub async fn api_v1_gesetzesvorhaben_gesvh_id_get(
    server: &LTZFServer,
    path_params: models::ApiV1GesetzesvorhabenGesvhIdGetPathParams,
) -> Result<models::Gesetzesvorhaben> {
    use crate::db::schema;
    let connection = server.database.get().await?;
    let gsid = connection.interact(move |conn| 
        schema::gesetzesvorhaben::table
        .filter(schema::gesetzesvorhaben::api_id.eq(path_params.gesvh_id))
        .select(schema::gesetzesvorhaben::id)
        .first::<i32>(conn))
        .await??;

    Ok(crate::db::retrieve::gsvh_by_id(gsid, &connection).await?)
}

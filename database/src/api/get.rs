use crate::{LTZFServer, Result};
use diesel::prelude::*;
use openapi::models;

pub async fn api_v1_gesetzesvorhaben_gesvh_id_get(
    server: &LTZFServer,
    path_params: models::ApiV1GesetzesvorhabenGesvhIdGetPathParams,
) -> Result<models::Gesetzesvorhaben> {
    tracing::info!("api_v1_gesetzesvorhaben_gesvh_id_get called");
    use crate::db::schema;
    let connection = server.database.get().await?;
    let gsid = connection
        .interact(move |conn| {
            schema::gesetzesvorhaben::table
                .filter(schema::gesetzesvorhaben::api_id.eq(path_params.gesvh_id))
                .select(schema::gesetzesvorhaben::id)
                .first::<i32>(conn)
        })
        .await??;

    Ok(crate::db::retrieve::gsvh_by_id(gsid, &connection).await?)
}

pub async fn api_v1_gesetzesvorhaben_get(
    server: &LTZFServer,
    query_params: models::ApiV1GesetzesvorhabenGetQueryParams,
) -> Result<models::Response> {
    tracing::info!("api_v1_gesetzesvorhaben_get called");
    let connection = server.database.get().await?;
    let gsvh = crate::db::retrieve::gsvh_by_parameter(query_params, &connection).await?;
    Ok(models::Response {
        payload: if gsvh.is_empty() { None } else { Some(gsvh) },
    })
}

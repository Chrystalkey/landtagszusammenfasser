use std::sync::Arc;

use crate::{infra::api::{self as ifapi, CUPPayload}, AppState};
use crate::infra::db::connection as dbcon;
use diesel_interaction::*;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

async fn update_gesvh(
    State(app): State<Arc<AppState>>,
    Json(cupdate): Json<ifapi::CUPUpdate>,
) -> impl IntoResponse {
    match &cupdate.payload{
        CUPPayload::GesVH(gesvh) => {
            let conn = app.pool.get().await;
            let conn = if let Err(conn) = conn{
                return (StatusCode::INTERNAL_SERVER_ERROR, "Could not Connect to Database").into_response();
            }else{
                conn.unwrap()
            };// TODO: implement it
            // if no match: create
            // if one match: update
            // if more than one match: error on ambiguous data, let a human decide
        },
        _ => {return (StatusCode::BAD_REQUEST, "This Endpoint is only for updating Gesetzesvorhaben")
                .into_response()}
    }
    (StatusCode::OK, Json(cupdate)).into_response()
}
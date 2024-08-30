use std::sync::Arc;

use crate::{infra::api as ifapi, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

fn update_gesvh(
    State(app): State<Arc<AppState>>,
    Json(gesvh): Json<ifapi::CUPUpdate>,
) -> impl IntoResponse {
    // if no match: create
    // if one match: update
    // if more than one match: error on ambiguous data, let a human decide
    (StatusCode::OK, Json(gesvh)).into_response()
}
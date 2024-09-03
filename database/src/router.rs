use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};


use crate::handlers::authentication::authenticate_collector;
use crate::infra::api::CUPResponse;
use crate::AppState;
use crate::infra::api as ifapi;
use crate::error::{LTZFError, Result};

pub fn app_router(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/api/v1/collector/:collector_id/gesetzesvorhaben", post(handle_gesvh))
        .fallback(handler_404)
}

async fn root() -> &'static str {
    "Server is running!"
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The requested resource was not found",
    )
}


async fn handle_gesvh(
    Path(collector_id): Path<String>,
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(cupdate): Json<ifapi::CUPUpdate>,
) -> std::result::Result<Json<CUPResponse>, LTZFError> {
    if !authenticate_collector(&headers, app.clone()).await? {
        return Err(crate::error::LTZFError::Unauthorized("Collector not authorized".to_owned()));
    }
    let response = crate::handlers::gesetzesvorhaben::handle_gesvh(app, cupdate).await?;
    Ok(Json(response))}

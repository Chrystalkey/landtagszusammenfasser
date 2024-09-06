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
use crate::error::*;

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
    let coll_id = uuid::Uuid::parse_str(collector_id.as_str()).map_err(ParsingError::from)?;
    authenticate_collector(coll_id,&headers, app.clone()).await?;
    tracing::info!("Collector {} called post(gesetzesvorhaben) with msg_id {}", coll_id, cupdate.msg_id);
    tracing::debug!("Received CUPUpdate Struct: {:?}", cupdate);
    tracing::debug!("headers: {:?}", headers);

    let response = crate::handlers::gesetzesvorhaben::handle_gesvh(app, cupdate).await?;
    Ok(Json(response))
}

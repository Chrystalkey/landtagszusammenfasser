use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};

use crate::handlers::authentication::authenticate_collector;
use crate::infra::api::CUPResponse;
use crate::AppState;
use crate::infra::api as ifapi;
use crate::error::*;

/// Dingens: Nur ein Endpoint für gesetzesvorhaben und assoziierte dinge inklusive Dokumente
pub fn app_router(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    let route = Router::new()
        .route("/", get(root))
        .route("/api/v1/collector/:collector_id/gesetzesvorhaben", post(post_gesvh))
        .route("/api/v1/collector/:collector_id/gesetzesvorhaben/:gesvh_id", put(put_gesvh));
    route_webservice(route)
        .fallback(handler_404)
}

pub fn route_webservice(router: Router<Arc<AppState>>) -> Router<Arc<AppState>>{
    router
    .route("/api/v1/webservice/gesetzesvorhaben/", get(get_gesvh))
    .route("/api/v1/webservice/gesetzesvorhaben/:gesvh_id", get(get_gesvh))
    .route("/api/v1/webservice/dokumente", get(get_dokument))
    .route("/api/v1/webservice/dokumente/:dok_id", get(get_dokument))
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

async fn get_dokument(
    Path(gesvh): Path<String>,
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
)-> Result<Json<ifapi::CUPResponse>> {
    todo!()
}

async fn get_gesvh(
    Path(gesvh): Path<String>,
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
)-> Result<Json<ifapi::CUPResponse>> {
    todo!()
}

async fn put_gesvh(
    Path(path_vars): Path<HashMap<String,String>>,
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(cupdate): Json<ifapi::CUPUpdate>,
) -> Result<Json<CUPResponse>> {
    let coll_id = uuid::Uuid::parse_str(path_vars.get("collector_id").unwrap()).map_err(ParsingError::from)?;
    let gesvh_id = uuid::Uuid::parse_str(path_vars.get("gesvh_id").unwrap()).map_err(ParsingError::from)?;

    authenticate_collector(coll_id,&headers, app.clone()).await?;
    tracing::info!("Collector {} called put(gesetzesvorhaben) with msg_id {} on Gesetzesvorhaben {}", coll_id, cupdate.msg_id, gesvh_id);
    tracing::debug!("Received CUPUpdate Struct: {:?}", cupdate);
    tracing::debug!("headers: {:?}", headers);
    let response = crate::handlers::gesetzesvorhaben::put_gesvh(app, cupdate, gesvh_id).await?;
    Ok(Json(response))
}

async fn post_gesvh(
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

    let response = crate::handlers::gesetzesvorhaben::post_gesvh(app, cupdate).await?;
    Ok(Json(response))
}

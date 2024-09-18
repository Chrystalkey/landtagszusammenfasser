use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::*;
use crate::handlers::authentication::authenticate_collector;
use crate::infra::api;
use crate::AppState;

pub fn app_router(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    let route = Router::new().route("/", get(root));
    let route = route_collector(route);
    route_webservice(route).fallback(handler_404)
}

pub fn route_collector(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route(
            "/api/v1/gesetzesvorhaben",
            post(post_gesvh),
        )
        .route(
            "/api/v1/gesetzesvorhaben/:gesvh_id",
            put(put_gesvh),
        )
}
pub fn route_webservice(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route(
            "/api/v1/webservice/gesetzesvorhaben/",
            get(get_gesvh_filter),
        )
        .route(
            "/api/v1/webservice/gesetzesvorhaben/:gesvh_id",
            get(get_gesvh),
        )
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

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct GetGesvhQueryFilters {
    pub updated_since: Option<DateTime<Utc>>,
    pub updated_until: Option<DateTime<Utc>>,
    pub created_since: Option<DateTime<Utc>>,
    pub created_until: Option<DateTime<Utc>>,
    pub parlament: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
}

/// GET /api/v1/webservice/gesetzesvorhaben?since=timestamp&until=timestamp&limit=number...
async fn get_gesvh_filter(
    State(app): State<Arc<AppState>>,
    Query(params): Query<GetGesvhQueryFilters>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    tracing::info!("Webservice API called GET gesetzesvorhaben without uuid");
    tracing::debug!("Received Query Parameters: {:?}", params);
    tracing::debug!("headers: {:?}", headers);
    let response = crate::handlers::gesetzesvorhaben::get_gesvh_filtered(app, params).await?;
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}

/// GET /api/v1/webservice/gesetzesvorhaben/:gesvh_id
async fn get_gesvh(
    State(app): State<Arc<AppState>>,
    Path(gesvh): Path<String>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    let gesvh_id = uuid::Uuid::parse_str(gesvh.as_str()).map_err(ParsingError::from)?;
    tracing::info!(
        "Webservice API called GET gesetzesvorhaben on Gesetzesvorhaben {}",
        gesvh_id
    );
    tracing::debug!("headers: {:?}", headers);
    let response = crate::handlers::gesetzesvorhaben::get_gesvh(app, gesvh_id).await?;
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}

/// PUT /api/v1/collector/gesetzesvorhaben/:gesvh_id?collector_id=uuid
/// All parts are mandatory, this is the only currently implemented end point
async fn put_gesvh(
    State(app): State<Arc<AppState>>,
    Path(path_vars): Path<HashMap<String, String>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(cupdate): Json<api::CUPUpdate>,
) -> Result<Json<api::CUPResponse>> {
    let coll_id =
        uuid::Uuid::parse_str(params.get("collector_id").unwrap()).map_err(ParsingError::from)?;
    let gesvh_id =
        uuid::Uuid::parse_str(path_vars.get("gesvh_id").unwrap()).map_err(ParsingError::from)?;

    authenticate_collector(coll_id, &headers, app.clone()).await?;
    tracing::info!(
        "Collector {} called put(gesetzesvorhaben) with msg_id {} on Gesetzesvorhaben {}",
        coll_id,
        cupdate.msg_id,
        gesvh_id
    );
    tracing::debug!("Received CUPUpdate Struct: {:?}", cupdate);
    tracing::debug!("headers: {:?}", headers);
    let response = crate::handlers::gesetzesvorhaben::put_gesvh(app, cupdate, gesvh_id).await?;
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}

/// POST /api/v1/collector/gesetzesvorhaben?collector_id=uuid
/// All parts are mandatory, this is the only currently implemented end point
async fn post_gesvh(
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(cupdate): Json<api::CUPUpdate>
) -> std::result::Result<StatusCode, LTZFError> {
    let coll_id = uuid::Uuid::parse_str(
        headers.get("collector_id").unwrap().to_str()
        .map_err(ParsingError::from)?
    )
    .map_err(ParsingError::from)?;
    authenticate_collector(coll_id, &headers, app.clone()).await?;
    tracing::info!(
        "Collector {} called post(gesetzesvorhaben) with msg_id {}",
        coll_id,
        cupdate.msg_id
    );
    tracing::debug!("Received CUPUpdate Struct: {:?}", cupdate);
    tracing::debug!("headers: {:?}", headers);

    let response = crate::handlers::gesetzesvorhaben::post_gesvh(app, cupdate).await?;
    tracing::debug!("Response: {:?}", response);
    Ok(response)
}

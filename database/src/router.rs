use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};

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
}
pub fn route_webservice(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route(
            "/api/v1/gesetzesvorhaben",
            get(get_gesvh_filter),
        )
        .route(
            "/api/v1/gesetzesvorhaben/:api_id",
            get(get_gesvh),
        )

        .route(
            "/api/v1/dokumente/:api_id",
            get(get_dok),
        )
        .route(
            "/api/v1/dokumente",
            get(get_dok_filter),
        )

        .route(
            "/api/v1/stationen/:api_id",
            get(get_stationen),
        )
        .route(
            "/api/v1/stationen",
            get(get_stationen_filter),
        )

        .route(
            "/api/v1/ausschuesse/:api_id",
            get(get_ausschuss),
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

async fn get_ausschuss(
    State(app): State<Arc<AppState>>,
    Path(ausschuss): Path<String>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    let ausschuss_id = uuid::Uuid::parse_str(ausschuss.as_str()).map_err(ParsingError::from)?;
    tracing::info!("Webservice API called GET ausschuesse on Ausschuss {}", ausschuss_id);
    tracing::debug!("headers: {:?}", headers);
    let response = todo!();
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}

async fn get_stationen(
    State(app): State<Arc<AppState>>,
    Path(station): Path<String>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    let station_id = uuid::Uuid::parse_str(station.as_str()).map_err(ParsingError::from)?;
    tracing::info!("Webservice API called GET stationen on Station {}", station_id);
    tracing::debug!("headers: {:?}", headers);
    let response = todo!();
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}
async fn get_stationen_filter(
    State(app): State<Arc<AppState>>,
    Query(params): Query<filters::StationFilter>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    tracing::info!("Webservice API called GET stationen without uuid");
    tracing::debug!("Received Query Parameters: {:?}", params);
    tracing::debug!("headers: {:?}", headers);
    let response = todo!();
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}
async fn get_dok_filter(
    State(app): State<Arc<AppState>>,
    Query(params): Query<filters::DokFilter>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    tracing::info!("Webservice API called GET dokumente without uuid");
    tracing::debug!("Received Query Parameters: {:?}", params);
    tracing::debug!("headers: {:?}", headers);
    let response = todo!();
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}
async fn get_dok(
    State(app): State<Arc<AppState>>,
    Path(dok): Path<String>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    let dok_id = uuid::Uuid::parse_str(dok.as_str()).map_err(ParsingError::from)?;
    tracing::info!("Webservice API called GET dokumente on Dokument {}", dok_id);
    tracing::debug!("headers: {:?}", headers);
    let response = todo!();
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}

/// GET /api/v1/webservice/gesetzesvorhaben?since=timestamp&until=timestamp&limit=number...
async fn get_gesvh_filter(
    State(app): State<Arc<AppState>>,
    Query(params): Query<filters::GesVHFilter>,
    headers: HeaderMap,
) -> Result<Json<api::WSResponse>> {
    tracing::info!("Webservice API called GET gesetzesvorhaben without uuid");
    tracing::debug!("Received Query Parameters: {:?}", params);
    tracing::debug!("headers: {:?}", headers);
    let response = crate::handlers::read::get_gesvh_filtered(app, params).await?;
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
    let response = crate::handlers::read::get_gesvh(app, gesvh_id).await?;
    tracing::debug!("Response: {:?}", response);
    Ok(Json(response))
}

/// POST /api/v1/collector/gesetzesvorhaben?collector_id=uuid
/// All parts are mandatory, this is the only currently implemented end point
async fn post_gesvh(
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(param) : Query<HashMap<String,String>>,
    Json(cupdate): Json<api::CUPUpdate>
) -> std::result::Result<StatusCode, LTZFError> {
    let coll_id = uuid::Uuid::parse_str(
        param.get("collector").unwrap()
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

    let response = crate::handlers::write::post_gesvh(app, cupdate).await?;
    tracing::debug!("Response: {:?}", response);
    Ok(response)
}

pub(crate) mod filters{
    use serde::Deserialize;
    #[derive(Debug, Clone, Deserialize)]
    pub struct GesVHFilter{
        pub updated_since: Option<chrono::DateTime<chrono::Utc>>,
        pub updated_until: Option<chrono::DateTime<chrono::Utc>>,
        pub limit: Option<usize>,
        pub offset: Option<usize>,
        pub parlament: Option<String>,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct DokFilter{
        pub since: Option<chrono::DateTime<chrono::Utc>>,
        pub until: Option<chrono::DateTime<chrono::Utc>>,
        pub limit: Option<usize>,
        pub offset: Option<usize>,
        pub typ: Option<String>,
        pub autor: Option<String>,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct StationFilter{
        pub since: Option<chrono::DateTime<chrono::Utc>>,
        pub until: Option<chrono::DateTime<chrono::Utc>>,
        pub limit: Option<usize>,
        pub offset: Option<usize>,
        pub status: Option<String>,
    }
}
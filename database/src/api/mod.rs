use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use diesel::Connection;
use lettre::SmtpTransport;

use crate::error::LTZFError;
use crate::Configuration;
use axum_extra::extract::CookieJar;
use deadpool_diesel::postgres::Pool;
use openapi::apis::default::*;
use openapi::models;

mod post;
mod auth;

pub struct LTZFServer {
    pub database: Pool,
    pub mailer: Option<SmtpTransport>,
    pub config: Configuration,
}
pub type LTZFArc = std::sync::Arc<LTZFServer>;
impl LTZFServer {
    pub fn new(database: Pool, mailer: Option<SmtpTransport>, config: Configuration) -> Self {
        Self {
            database,
            mailer,
            config,
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl openapi::apis::default::Default for LTZFServer {
    /// ApiV1GesetzesvorhabenGesvhIdGet - GET /api/v1/gesetzesvorhaben/{gesvh_id}
    async fn api_v1_gesetzesvorhaben_gesvh_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ApiV1GesetzesvorhabenGesvhIdGetPathParams,
    ) -> Result<ApiV1GesetzesvorhabenGesvhIdGetResponse, String> {
        Ok(todo!())
    }

    /// ApiV1GesetzesvorhabenGet - GET /api/v1/gesetzesvorhaben
    async fn api_v1_gesetzesvorhaben_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: models::ApiV1GesetzesvorhabenGetQueryParams,
    ) -> Result<ApiV1GesetzesvorhabenGetResponse, String> {
        Ok(todo!())
    }

    /// ApiV1GesetzesvorhabenPost - POST /api/v1/gesetzesvorhaben
    async fn api_v1_gesetzesvorhaben_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: models::ApiV1GesetzesvorhabenPostQueryParams,
        body: models::Gesetzesvorhaben,
    ) -> Result<ApiV1GesetzesvorhabenPostResponse, String> {
        auth::authenticate().map_err(|e| e.to_string())?;
        post::api_v1_gesetzesvorhaben_post(self, body).await
        .map_err(|e| {
            tracing::warn!("Error Occurred and Is Returned: {:?}", e);e.to_string()})?;
        Ok(ApiV1GesetzesvorhabenPostResponse::Status201_SuccessfullyCreatedOrIntegratedTheObject)
    }
}

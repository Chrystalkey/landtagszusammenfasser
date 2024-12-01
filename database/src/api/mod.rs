use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use lettre::SmtpTransport;

use crate::Configuration;
use axum_extra::extract::CookieJar;
use deadpool_diesel::postgres::Pool;
use openapi::apis::default::*;
use openapi::models;

mod post;

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
        path_params: models::ApiV1GesetzesvorhabenPostPathParams,
        body: models::Gesetzesvorhaben,
    ) -> Result<ApiV1GesetzesvorhabenPostResponse, String> {
        Ok(todo!())
    }
}

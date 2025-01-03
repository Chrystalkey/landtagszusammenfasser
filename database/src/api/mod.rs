use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use lettre::SmtpTransport;

use crate::error::{DataValidationError, DatabaseError, LTZFError};
use crate::Configuration;
use axum_extra::extract::CookieJar;
use deadpool_diesel::postgres::Pool;
use openapi::apis::default::*;
use openapi::models;

mod auth;
mod get;
mod post;
mod put;

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
    type Claims = ();

    #[doc = " ApiV1GesetzesvorhabenGesvhIdGet - GET /api/v1/gesetzesvorhaben/{gsvh_id}"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn api_v1_gesetzesvorhaben_gsvh_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
          header_params: models::ApiV1GesetzesvorhabenGsvhIdGetHeaderParams,
          path_params: models::ApiV1GesetzesvorhabenGsvhIdGetPathParams,
        ) -> Result<ApiV1GesetzesvorhabenGsvhIdGetResponse, ()> {
        tracing::info!(
            "Get By ID endpoint called with ID: {}",
            path_params.gsvh_id
        );
        let gsvh = get::api_v1_gesetzesvorhaben_gesvh_id_get(self, path_params).await;

        match gsvh {
            Ok(gsvh) => {
                Ok(ApiV1GesetzesvorhabenGsvhIdGetResponse::Status200_SuccessfulOperation(gsvh))
            }
            Err(e) => {
                tracing::warn!("{}", e.to_string());
                match e {
                    LTZFError::Database{source: DatabaseError::Operation{source: diesel::result::Error::NotFound}} => {
                        tracing::warn!("Not Found Error: {:?}", e.to_string());
                        Ok(ApiV1GesetzesvorhabenGsvhIdGetResponse::Status404_ContentNotFound)
                    }
                    _ => Err(()),
                }
            }
        }
    }

    #[doc = " ApiV1GesetzesvorhabenGet - GET /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn api_v1_gesetzesvorhaben_gsvh_id_put(
            &self,
            method: Method,
            host: Host,
            cookies: CookieJar,
            claims: Self::Claims,
          path_params: models::ApiV1GesetzesvorhabenGsvhIdPutPathParams,
        ) -> Result<ApiV1GesetzesvorhabenGsvhIdPutResponse, ()> {
            tracing::debug!("api_v1_gesetzesvorhaben_gsvh_id_put Called with path params: `{:?}`", path_params);
            let out = put::api_v1_gesetzesvorhaben_gsvh_id_put(self, path_params)
            .await
            .map_err(|e| todo!())?;
            Ok(out)
        }

    #[doc = " ApiV1GesetzesvorhabenGet - GET /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn api_v1_gesetzesvorhaben_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
          header_params: models::ApiV1GesetzesvorhabenGetHeaderParams,
          query_params: models::ApiV1GesetzesvorhabenGetQueryParams,
        ) -> Result<ApiV1GesetzesvorhabenGetResponse, ()> {
        tracing::info!(
            "GET GSVHByParam endpoint called with query params: {:?}",
            query_params
        );
        match get::api_v1_gesetzesvorhaben_get(self, query_params, header_params).await {
            Ok(models::Response{payload: None}) => Ok(ApiV1GesetzesvorhabenGetResponse::Status204_NoContentFoundForTheSpecifiedParameters),
            Ok(x) => Ok(ApiV1GesetzesvorhabenGetResponse::Status200_SuccessfulOperation(x)),
            Err(e) => {
                tracing::warn!("{}", e.to_string());
                Err(())
            }
        }
    }

    #[doc = " ApiV1GesetzesvorhabenPost - POST /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn api_v1_gesetzesvorhaben_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
            claims: Self::Claims,
          query_params: models::ApiV1GesetzesvorhabenPostQueryParams,
                body: models::Gesetzesvorhaben,
        ) -> Result<ApiV1GesetzesvorhabenPostResponse, ()> {
        tracing::info!("api_v1_gesetzesvorhaben_post called by {:?}", query_params);

        let rval = post::api_v1_gesetzesvorhaben_post(self, body).await;
        match rval {
            Ok(_) => {
                Ok(ApiV1GesetzesvorhabenPostResponse::Status201_SuccessfullyIntegratedTheObject)
            }
            Err(e) => {
                tracing::warn!("Error Occurred and Is Returned: {:?}", e.to_string());
                match e {
                    LTZFError::Database{ source: DatabaseError::Operation{source: diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        info,
                    )}} => {
                        tracing::warn!(
                            "Unique Violation Error (Conflict on Input Data): {:?}",
                            info
                        );
                        Ok(ApiV1GesetzesvorhabenPostResponse::Status409_Conflict)
                    },
                    LTZFError::Validation{source: DataValidationError::DuplicateApiId{id}} => {
                        tracing::warn!("ApiID Equal Error: {:?}", id);
                        Ok(ApiV1GesetzesvorhabenPostResponse::Status409_Conflict)
                    },
                    LTZFError::Validation{source: DataValidationError::AmbiguousMatch{..}} =>{
                        Ok(ApiV1GesetzesvorhabenPostResponse::Status409_Conflict)
                    }
                    _ => {
                        Err(())
                    },
                }
            }
        }
    }
}

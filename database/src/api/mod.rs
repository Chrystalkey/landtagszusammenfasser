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
    type Claims = (auth::APIScope, i32);

    #[doc = "AuthGet - GET /api/v1/auth"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn auth_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        body: models::CreateApiKey,
    ) -> Result<AuthGetResponse, ()> {
        if claims.0 != auth::APIScope::KeyAdder {
            return Ok(AuthGetResponse::Status401_APIKeyIsMissingOrInvalid { www_authenticate: None })
        }
        let key = auth::auth_get(self, body.scope.try_into().unwrap(), body.expires_at.map(|x| x.naive_utc()), claims.1).await;
        match key{
            Ok(key) => {return Ok(AuthGetResponse::Status201_APIKeyWasCreatedSuccessfully(key))}
            Err(e) =>{
                tracing::error!("{}", e.to_string());
                return Ok(AuthGetResponse::Status500_InternalServerError);
            }
        }
    }

    #[doc = "AuthDelete - DELETE /api/v1/auth"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn auth_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        body: String,
    ) -> Result<AuthDeleteResponse, ()> {
        let ret = auth::auth_delete(self, claims.0, &body).await;
        match ret {
            Ok(x) => {return Ok(x)},
            Err(e) => {
                tracing::error!("{}", e.to_string());
                Err(())
            }
        }
    }

    #[doc = "GsvhGetById - GET /api/v1/gesetzesvorhaben/{gsvh_id}"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn gsvh_get_by_id(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
          header_params: models::GsvhGetByIdHeaderParams,
          path_params: models::GsvhGetByIdPathParams,
        ) -> Result<GsvhGetByIdResponse, ()> {
        tracing::info!(
            "Get By ID endpoint called with ID: {}",
            path_params.gsvh_id
        );
        let gsvh = get::api_v1_gesetzesvorhaben_gesvh_id_get(self, path_params).await;

        match gsvh {
            Ok(gsvh) => {
                Ok(GsvhGetByIdResponse::Status200_SuccessfulOperation(gsvh))
            }
            Err(e) => {
                tracing::warn!("{}", e.to_string());
                match e {
                    LTZFError::Database{source: DatabaseError::Operation{source: diesel::result::Error::NotFound}} => {
                        tracing::warn!("Not Found Error: {:?}", e.to_string());
                        Ok(GsvhGetByIdResponse::Status404_ContentNotFound)
                    }
                    _ => Err(()),
                }
            }
        }
    }

    #[doc = " GsvhGet - GET /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn gsvh_put(
            &self,
            method: Method,
            host: Host,
            cookies: CookieJar,
            claims: Self::Claims,
            path_params: models::GsvhPutPathParams,
            body: models::Gesetzesvorhaben,
        ) -> Result<GsvhPutResponse, ()> {
            tracing::trace!("api_v1_gesetzesvorhaben_gsvh_id_put Called with path params: `{:?}`", path_params);
            let out = put::api_v1_gesetzesvorhaben_gsvh_id_put(self, path_params, body)
            .await
            .map_err(|e| todo!())?;
            Ok(out)
        }

    #[doc = " GsvhGet - GET /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn gsvh_get(
            &self,
            method: Method,
            host: Host,
            cookies: CookieJar,
            query_params: models::GsvhGetQueryParams,
        ) -> Result<GsvhGetResponse, ()> {
        tracing::trace!(
            "GET GSVHByParam endpoint called with query params: {:?}",
            query_params
        );
        match get::api_v1_gesetzesvorhaben_get(self, query_params).await {
            Ok(models::Response{payload: None}) => Ok(GsvhGetResponse::Status204_NoContentFoundForTheSpecifiedParameters),
            Ok(x) => Ok(GsvhGetResponse::Status200_SuccessfulOperation(x)),
            Err(e) => {
                tracing::warn!("{}", e.to_string());
                Err(())
            }
        }
    }

    #[doc = " ApiV1GesetzesvorhabenPost - POST /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn gsvh_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
            claims: Self::Claims,
          query_params: models::GsvhPostQueryParams,
                body: models::Gesetzesvorhaben,
        ) -> Result<GsvhPostResponse, ()> {
        tracing::trace!("api_v1_gesetzesvorhaben_post called by {:?}", query_params);

        let rval = post::api_v1_gesetzesvorhaben_post(self, body).await;
        match rval {
            Ok(_) => {
                Ok(GsvhPostResponse::Status201_SuccessfullyIntegratedTheObject)
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
                        Ok(GsvhPostResponse::Status409_Conflict)
                    },
                    LTZFError::Validation{source: DataValidationError::DuplicateApiId{id}} => {
                        tracing::warn!("ApiID Equal Error: {:?}", id);
                        Ok(GsvhPostResponse::Status409_Conflict)
                    },
                    LTZFError::Validation{source: DataValidationError::AmbiguousMatch{..}} =>{
                        Ok(GsvhPostResponse::Status409_Conflict)
                    }
                    _ => {
                        Err(())
                    },
                }
            }
        }
    }
}

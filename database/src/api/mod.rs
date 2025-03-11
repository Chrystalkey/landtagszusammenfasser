use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use lettre::SmtpTransport;

use crate::db::delete::delete_ass_by_api_id;
use crate::error::{DataValidationError, DatabaseError, LTZFError};
use crate::{db, Configuration};
use axum_extra::extract::CookieJar;
use openapi::apis::default::*;
use openapi::models;

mod auth;
mod get;
mod put;

#[derive(Clone)]
pub struct LTZFServer {
    pub sqlx_db: sqlx::PgPool,
    pub mailer: Option<SmtpTransport>,
    pub config: Configuration,
}
pub type LTZFArc = std::sync::Arc<LTZFServer>;
impl LTZFServer {
    pub fn new(
        sqlx_db: sqlx::PgPool,
        mailer: Option<SmtpTransport>,
        config: Configuration,
    ) -> Self {
        Self {
            mailer,
            config,
            sqlx_db,
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl openapi::apis::default::Default for LTZFServer {
    type Claims = (auth::APIScope, i32);

    #[doc = "AuthPost - POST /api/v1/auth"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn auth_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        body: models::CreateApiKey,
    ) -> Result<AuthPostResponse, ()> {
        if claims.0 != auth::APIScope::KeyAdder {
            return Ok(AuthPostResponse::Status401_APIKeyIsMissingOrInvalid);
        }
        let key = auth::auth_get(
            self,
            body.scope.try_into().unwrap(),
            body.expires_at.map(|x| x),
            claims.1,
        )
        .await;
        match key {
            Ok(key) => {
                return Ok(AuthPostResponse::Status201_APIKeyWasCreatedSuccessfully(
                    key,
                ))
            }
            Err(e) => {
                tracing::error!("{}", e.to_string());
                return Err(());
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
        header_params: models::AuthDeleteHeaderParams,
    ) -> Result<AuthDeleteResponse, ()> {
        let key_to_delete = header_params.api_key_delete;
        let ret = auth::auth_delete(self, claims.0, &key_to_delete).await;
        match ret {
            Ok(x) => return Ok(x),
            Err(e) => {
                tracing::error!("{}", e.to_string());
                Err(())
            }
        }
    }

    #[doc = "VorgangGetById - GET /api/v1/vorgang/{vorgang_id}"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn vorgang_get_by_id(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        header_params: models::VorgangGetByIdHeaderParams,
        path_params: models::VorgangGetByIdPathParams,
    ) -> Result<VorgangGetByIdResponse, ()> {
        let vorgang = get::vg_id_get(self, header_params, path_params).await;

        match vorgang {
            Ok(vorgang) => Ok(VorgangGetByIdResponse::Status200_SuccessfulOperation(
                vorgang,
            )),
            Err(e) => {
                tracing::warn!("{}", e.to_string());
                match e {
                    LTZFError::Database {
                        source:
                            DatabaseError::Sqlx {
                                source: sqlx::Error::RowNotFound,
                            },
                    } => {
                        tracing::warn!("Not Found Error: {:?}", e.to_string());
                        Ok(VorgangGetByIdResponse::Status404_ContentNotFound)
                    }
                    _ => Err(()),
                }
            }
        }
    }
    #[doc = " VorgangDelete - GET /api/v1/vorgang"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn vorgang_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        path_params: models::VorgangDeletePathParams,
    ) -> Result<VorgangDeleteResponse, ()> {
        if claims.0 != auth::APIScope::Admin && claims.0 != auth::APIScope::KeyAdder {
            return Ok(VorgangDeleteResponse::Status401_APIKeyIsMissingOrInvalid);
        }
        let api_id = path_params.vorgang_id;
        let result = db::delete::delete_vorgang_by_api_id(api_id, self)
            .await
            .map_err(|e| {
                tracing::warn!("Could not delete Vorgang with ID `{}`: {}", api_id, e);
            });
        return result;
    }
    #[doc = " VorgangIdPut - GET /api/v1/vorgang"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn vorgang_id_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        path_params: models::VorgangIdPutPathParams,
        body: models::Vorgang,
    ) -> Result<VorgangIdPutResponse, ()> {
        if claims.0 != auth::APIScope::Admin && claims.0 != auth::APIScope::KeyAdder {
            return Ok(VorgangIdPutResponse::Status401_APIKeyIsMissingOrInvalid);
        }
        let out = put::api_v1_vorgang_id_put(self, path_params, body)
            .await
            .map_err(|e| tracing::warn!("{}", e))?;
        Ok(out)
    }

    #[doc = " VorgangGet - GET /api/v1/vorgang"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn vorgang_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        header_params: models::VorgangGetHeaderParams,
        query_params: models::VorgangGetQueryParams,
    ) -> Result<VorgangGetResponse, ()> {
        let now = chrono::Utc::now();
        let lowest_upd_bound = if query_params.upd_since.is_some() && header_params.if_modified_since.is_some(){
            query_params.upd_since.unwrap().min(header_params.if_modified_since.unwrap())
        }else{
            query_params.upd_since.unwrap_or(
                header_params.if_modified_since.unwrap_or(
                    chrono::DateTime::parse_from_rfc3339("1940-01-01T00:00:00").unwrap().to_utc()
                )
            )
        };

        if  lowest_upd_bound > now || (query_params.upd_until.is_some() && query_params.upd_until.unwrap() < lowest_upd_bound) {
            return Ok(VorgangGetResponse::Status416_RequestRangeNotSatisfiable);
        }
        match get::vg_get(self, header_params, query_params).await {
            Ok(models::VorgangGet200Response { payload: None }) => {
                Ok(VorgangGetResponse::Status204_NoContentFoundForTheSpecifiedParameters)
            }
            Ok(x) => Ok(VorgangGetResponse::Status200_AntwortAufEineGefilterteAnfrageZuVorgang(x)),
            Err(e) => {
                tracing::warn!("{}", e.to_string());
                Err(())
            }
        }
    }

    #[doc = " ApiV1VorgangPost - PUT /api/v1/vorgang"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn vorgang_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        query_params: models::VorgangPutQueryParams,
        body: models::Vorgang,
    ) -> Result<VorgangPutResponse, ()> {
        let rval = put::api_v1_vorgang_put(self, body).await;
        match rval {
            Ok(_) => Ok(VorgangPutResponse::Status201_SuccessfullyIntegratedTheObject),
            Err(e) => {
                tracing::warn!("Error Occurred and Is Returned: {:?}", e.to_string());
                match e {
                    LTZFError::Validation {
                        source: DataValidationError::AmbiguousMatch { .. },
                    } => Ok(VorgangPutResponse::Status409_Conflict),
                    _ => Err(()),
                }
            }
        }
    }
    /// AsDelete - DELETE /api/v1/ausschusssitzung/{as_id}
    async fn as_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        path_params: models::AsDeletePathParams,
    ) -> Result<AsDeleteResponse, ()> {
        if claims.0 != auth::APIScope::Admin && claims.0 != auth::APIScope::KeyAdder {
            return Ok(AsDeleteResponse::Status401_APIKeyIsMissingOrInvalid);
        }
        Ok(delete_ass_by_api_id(path_params.as_id, self)
            .await
            .map_err(|e| {
                tracing::warn!("{}", e);
            })?)
    }

    /// AsGetById - GET /api/v1/ausschusssitzung/{as_id}
    async fn as_get_by_id(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        header_params: models::AsGetByIdHeaderParams,
        path_params: models::AsGetByIdPathParams,
    ) -> Result<AsGetByIdResponse, ()> {
        let ass = get::as_get_by_id(self,header_params, path_params).await.map_err(|e| {
            tracing::warn!("{}", e);
        })?;
        return Ok(ass);
    }

    /// AsIdPut - PUT /api/v1/ausschusssitzung/{as_id}
    async fn as_id_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        path_params: models::AsIdPutPathParams,
        body: models::Ausschusssitzung,
    ) -> Result<AsIdPutResponse, ()> {
        if claims.0 != auth::APIScope::Admin && claims.0 != auth::APIScope::KeyAdder {
            return Ok(AsIdPutResponse::Status401_APIKeyIsMissingOrInvalid);
        }
        let out = put::as_id_put(self, path_params, body).await.map_err(|e| {
            tracing::warn!("{}", e);
        })?;
        Ok(out)
    }

    /// AsPut - PUT /api/v1/ausschusssitzung
    async fn as_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        claims: Self::Claims,
        query_params: models::AsPutQueryParams,
        body: models::Ausschusssitzung,
    ) -> Result<AsPutResponse, ()> {
        let rval = put::as_put(self, body).await;
        match rval {
            Ok(_) => Ok(AsPutResponse::Status201_SuccessfullyIntegratedTheObject),
            Err(e) => {
                tracing::warn!("Error Occurred and Is Returned: {:?}", e.to_string());
                match e {
                    LTZFError::Validation {
                        source: DataValidationError::AmbiguousMatch { .. },
                    } => Ok(AsPutResponse::Status409_Konflikt),
                    _ => Err(()),
                }
            }
        }
    }

    /// AsGet - GET /api/v1/ausschusssitzung
    async fn as_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        header_params: models::AsGetHeaderParams,
        query_params: models::AsGetQueryParams,
    ) -> Result<AsGetResponse, ()> {
        let now = chrono::Utc::now();
        let lowest_upd_bound = if query_params.upd_since.is_some() && header_params.if_modified_since.is_some(){
            query_params.upd_since.unwrap().min(header_params.if_modified_since.unwrap())
        }else{
            query_params.upd_since.unwrap_or(
                header_params.if_modified_since.unwrap_or(
                    chrono::DateTime::parse_from_rfc3339("1940-01-01T00:00:00").unwrap().to_utc()
                )
            )
        };

        if  lowest_upd_bound > now || (query_params.upd_until.is_some() && query_params.upd_until.unwrap() < lowest_upd_bound) {
            return Ok(AsGetResponse::Status416_RequestRangeNotSatisfiable);
        }
        match get::as_get(self,header_params, query_params).await {
            Ok(models::AsGet200Response { payload: None }) => {
                Ok(AsGetResponse::Status204_NoContentFoundForTheSpecifiedParameters)
            }
            Ok(x) => {
                Ok(AsGetResponse::Status200_AntwortAufEineGefilterteAnfrageZuAusschusssitzungen(x))
            }
            Err(e) => {
                tracing::warn!("{}", e.to_string());
                Err(())
            }
        }
    }
}

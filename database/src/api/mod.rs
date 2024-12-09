use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use lettre::SmtpTransport;

use crate::Configuration;
use crate::error::LTZFError;
use axum_extra::extract::CookieJar;
use deadpool_diesel::postgres::Pool;
use openapi::apis::default::*;
use openapi::models;

mod post;
mod get;
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
    #[doc = " ApiV1GesetzesvorhabenGesvhIdGet - GET /api/v1/gesetzesvorhaben/{gesvh_id}"]
    #[must_use]
    #[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
    async fn api_v1_gesetzesvorhaben_gesvh_id_get(&self,
        method:Method,
        host:Host,
        cookies:CookieJar,
        path_params:models::ApiV1GesetzesvorhabenGesvhIdGetPathParams,) -> 
        Result<ApiV1GesetzesvorhabenGesvhIdGetResponse, String> {
            tracing::info!("Get By ID endpoint called with ID: {}", path_params.gesvh_id);
            let gsvh = get::api_v1_gesetzesvorhaben_gesvh_id_get(self, path_params).await;

            match gsvh {
                Ok(gsvh) => Ok(ApiV1GesetzesvorhabenGesvhIdGetResponse::Status200_SuccessfulOperation(gsvh)),
                Err(e) => {
                    tracing::warn!("{}", e.to_string());
                    match e{
                        LTZFError::DieselError(diesel::result::Error::NotFound) => {
                            tracing::warn!("Not Found Error: {:?}", e.to_string());
                            Ok(ApiV1GesetzesvorhabenGesvhIdGetResponse::Status404_ContentNotFound)
                        },
                        _ => Err(format!("{}", e))
                    }
                }
            }
    }
    
    #[doc = " ApiV1GesetzesvorhabenGet - GET /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
    async fn api_v1_gesetzesvorhaben_get(&self,
        method:Method,
        host:Host,
        cookies:CookieJar,
        query_params:models::ApiV1GesetzesvorhabenGetQueryParams,) ->  Result<ApiV1GesetzesvorhabenGetResponse, String>{
            todo!()
    }
    
    #[doc = " ApiV1GesetzesvorhabenPost - POST /api/v1/gesetzesvorhaben"]
    #[must_use]
    #[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
    async fn api_v1_gesetzesvorhaben_post(
        &self,method:Method,
        host:Host,
        cookies:CookieJar,
        query_params:models::ApiV1GesetzesvorhabenPostQueryParams,
        body:models::Gesetzesvorhaben,) ->  
        Result<ApiV1GesetzesvorhabenPostResponse, String>{
        auth::authenticate()
        .map_err(|e| {tracing::error!("{}", e.to_string()); format!("{}", e)})?;

        let rval = post::api_v1_gesetzesvorhaben_post(self, body).await;
        match rval {
            Ok(_) => Ok(ApiV1GesetzesvorhabenPostResponse::Status201_SuccessfullyIntegratedTheObject),
            Err(e) => {
                tracing::warn!("Error Occurred and Is Returned: {:?}", e.to_string());
                match e{
                    LTZFError::DieselError(diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                         info)) =>  {
                            tracing::warn!("Unique Violation Error (Conflict on Input Data): {:?}", info);
                            Ok(ApiV1GesetzesvorhabenPostResponse::Status409_Conflict)
                        },
                    _ => Err(format!("{}", e))
                }
            }
        }
    }
}

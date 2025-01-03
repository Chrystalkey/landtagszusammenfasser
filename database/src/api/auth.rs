use crate::{Result, LTZFServer};
use axum::async_trait;
use openapi::apis::ApiKeyAuthHeader;

#[async_trait]
impl ApiKeyAuthHeader for LTZFServer{
    type Claims = ();
    async fn extract_claims_from_header(& self, _headers: & axum::http::header::HeaderMap, _key: & str) ->  Option<Self::Claims> {
        tracing::warn!("Authentication is not implemented!");
        Some(())
    }
}

use crate::{error::LTZFError, LTZFServer, Result};
use axum::async_trait;
use openapi::apis::ApiKeyAuthHeader;
use sha256::digest;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum APIScope {
    KeyAdder,
    Admin,
    Collector,
}
impl TryFrom<&str> for APIScope{
    type Error = LTZFError;
    fn try_from(value: &str) -> Result<Self> {
        match value {
            "keyadder" => Ok(APIScope::KeyAdder),
            "admin" => Ok(APIScope::Admin),
            "collector" => Ok(APIScope::Collector),
            _ => Err(LTZFError::Validation{source: crate::error::DataValidationError::InvalidEnumValue { msg: format!("Tried to Convert {} to APIScope", value) }}),
        }
    }
}
impl TryFrom<String> for APIScope{
    type Error = LTZFError;
    fn try_from(value: String) -> Result<Self> {
        APIScope::try_from(value.as_str())
    }
}
impl ToString for APIScope{
    fn to_string(&self) -> String {
        match self {
            APIScope::KeyAdder => "keyadder".to_string(),
            APIScope::Admin => "admin".to_string(),
            APIScope::Collector => "collector".to_string(),
        }
    }
}
type ClaimType = (APIScope, i32);
async fn internal_extract_claims(server: &LTZFServer, headers: & axum::http::header::HeaderMap, key: & str) -> Result<ClaimType>{
    let key = headers.get(key);
    if key == None{
        return Err(LTZFError::Validation{source: crate::error::DataValidationError::MissingField { field: "X-API-Key".to_string() }});
    }
    let key = key.unwrap().to_str()?;
    let hash = digest(key);
    tracing::trace!("Authenticating Key Hash {}", hash);
    let table_rec = sqlx::query!(
        "SELECT api_keys.key_id, deleted, expires_at, value as scope FROM api_keys
    NATURAL LEFT JOIN api_scope WHERE key_hash = $1", hash)
    .map(|r|
        (r.key_id, r.deleted, r.expires_at, r.scope)
    )
    .fetch_optional(&server.sqlx_db).await?;

    tracing::trace!("DB Result: {:?}", table_rec);
    match table_rec{
        Some((_, true, _, _)) => {
            return Err(LTZFError::Validation { source:  crate::error::DataValidationError::Unauthorized { reason: format!("API Key was valid but is deleted. Hash: {}", hash) }});
        }
        Some((id, _, expires_at, scope)) => {
            if expires_at < chrono::Utc::now() {
                return Err(LTZFError::Validation { source: crate::error::DataValidationError::Unauthorized { reason: format!("API Key was valid but is expired. Hash: {}", hash) } });
            }
            let scope = (APIScope::try_from(scope.as_str()).unwrap(), id);
            sqlx::query!("UPDATE api_keys SET last_used = $1 WHERE key_hash = $2", chrono::Utc::now(), hash)
            .execute(&server.sqlx_db).await?;
            tracing::trace!("Scope of key with hash`{}`: {:?}", hash, scope.0);
            return Ok(scope)
        },
        None => {
            return Err(LTZFError::Validation { source: crate::error::DataValidationError::Unauthorized { reason: "API Key was not found in the Database".to_string() } });
        }
    }
}

#[async_trait]
impl ApiKeyAuthHeader for LTZFServer{
    type Claims = ClaimType;
    async fn extract_claims_from_header(& self, headers: & axum::http::header::HeaderMap, key: & str) ->  Option<Self::Claims> {
        match internal_extract_claims(self, headers, key).await {
            Ok(claim) => Some(claim),
            Err(error) => {tracing::warn!("Authorization failed: {}", error);None}
        }
    }
}

pub async fn auth_get(server: &LTZFServer, scope: APIScope, expires_at: Option<crate::DateTime>, created_by: i32) -> Result<String>{
    let key = generate_api_key().await;
    let key_digest = digest(key.clone());
    
    sqlx::query!("INSERT INTO api_keys(key_hash, created_by, expires_at, scope_id)
    VALUES
    ($1, $2, $3, (SELECT scope_id FROM api_scope WHERE value = $4))", 
    key_digest, created_by, expires_at.unwrap_or(chrono::Utc::now() + chrono::Duration::days(365)), scope.to_string()
    )
    .execute(&server.sqlx_db).await?;

    tracing::info!("Generated Fresh API Key with Scope: {:?}", scope);
    Ok(key)
}

pub async fn auth_delete(server: &LTZFServer, scope: APIScope, key: &str) -> Result<openapi::apis::default::AuthDeleteResponse>{
    if scope != APIScope::KeyAdder {
        tracing::warn!("Unauthorized: API Key does not have the required permission scope");
        return Ok(openapi::apis::default::AuthDeleteResponse::Status401_APIKeyIsMissingOrInvalid);
    }
    let hash = digest(key);
    let ret = sqlx::query!("UPDATE api_keys SET deleted=TRUE WHERE key_hash=$1 RETURNING key_id", hash)
    .fetch_optional(&server.sqlx_db).await?;

    if let Some(_) = ret {
        return Ok(openapi::apis::default::AuthDeleteResponse::Status204_APIKeyWasDeletedSuccessfully);
    }
    return Ok(openapi::apis::default::AuthDeleteResponse::Status404_APIKeyNotFound);
}

pub async fn generate_api_key() -> String{
    let key: String = "ltzf_".chars()
        .chain(thread_rng()
        .sample_iter(&Alphanumeric)
        .take(59)
        .map(char::from))
        .collect();
    key
}

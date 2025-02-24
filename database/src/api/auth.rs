use crate::{error::LTZFError, LTZFServer, Result};
use diesel::prelude::*;
use crate::db::schema;
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

    let connection = server.database.get().await?;

    let table_res = schema::api_keys::table
    .inner_join(schema::api_scope::table)
    .select((schema::api_keys::id, schema::api_keys::deleted, schema::api_keys::expires_at, schema::api_scope::api_key))
    .filter(schema::api_keys::key_hash.eq(hash.clone()));
    let table_res = connection.interact(|conn|{
        table_res.get_result::<(i32, bool, crate::DateTime, String)>(conn)
        .optional()
    }).await??;

    tracing::trace!("DB Result: {:?}", table_res);
    match table_res{
        Some((_, true, _, _)) => {
            return Err(LTZFError::Validation { source:  crate::error::DataValidationError::Unauthorized { reason: format!("API Key was valid but is deleted. Hash: {}", hash) }});
        }
        Some((id, _, expires_at, scope)) => {
            if expires_at < chrono::Utc::now() {
                return Err(LTZFError::Validation { source: crate::error::DataValidationError::Unauthorized { reason: format!("API Key was valid but is expired. Hash: {}", hash) } });
            }
            let scope = (APIScope::try_from(scope.as_str()).unwrap(), id);
            let stmt = diesel::update(schema::api_keys::table)
            .filter(schema::api_keys::key_hash.eq(hash.clone()))
            .set(schema::api_keys::last_used.eq(chrono::Utc::now()));
            connection.interact(|conn|{
                stmt.execute(conn)
            }).await??;
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
    let connection = server.database.get().await?;
    let scope_stmt = schema::api_scope::table.filter(schema::api_scope::api_key.eq(scope.to_string())).select(schema::api_scope::id);
    connection.interact(move |conn|{
        let scope_id = scope_stmt.first::<i32>(conn)?;
        diesel::insert_into(schema::api_keys::table)
        .values(
            (
            schema::api_keys::key_hash.eq(key_digest),
            schema::api_keys::scope.eq(scope_id),
            schema::api_keys::created_by.eq(created_by),
            schema::api_keys::expires_at.eq(expires_at.unwrap_or(chrono::Utc::now() + chrono::Duration::days(365))),
            )
        )
        .execute(conn)
    }).await??;
    tracing::info!("Generated Fresh API Key with Scope: {:?}", scope);
    Ok(key)
}

pub async fn auth_delete(server: &LTZFServer, scope: APIScope, key: &str) -> Result<openapi::apis::default::AuthDeleteResponse>{
    if scope != APIScope::KeyAdder {
        tracing::warn!("Unauthorized: API Key does not have the required permission scope");
        return Ok(openapi::apis::default::AuthDeleteResponse::Status401_APIKeyIsMissingOrInvalid);
    }
    let hash = digest(key);
    let connection = server.database.get().await?;
    let mut found = true;
    let _ = connection.interact(move|conn|{
        diesel::update(schema::api_keys::table.filter(schema::api_keys::key_hash.eq(hash)))
        .set(schema::api_keys::deleted.eq(true))
        .returning(schema::api_keys::id)
        .get_result::<i32>(conn)
    }).await?
    .map_err(|e: diesel::result::Error| {
        if e == diesel::result::Error::NotFound {
            found = false;
        }
        return e;
    })?;
    if found {
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

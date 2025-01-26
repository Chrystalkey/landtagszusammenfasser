use crate::{error::LTZFError, LTZFServer, Result};
use diesel::prelude::*;
use crate::db::schema;
use axum::async_trait;
use openapi::apis::ApiKeyAuthHeader;
use sha256::digest;

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

#[async_trait]
impl ApiKeyAuthHeader for LTZFServer{
    type Claims = APIScope;
    async fn extract_claims_from_header(& self, _headers: & axum::http::header::HeaderMap, key: & str) ->  Option<Self::Claims> {
        let hash = digest(key);

        let connection = self.database.get().await
        .map_err(|e| panic!("An Error Occurred trying to get a database connection: {}", e))
        .unwrap();
        let table_res = schema::api_keys::table
        .inner_join(schema::api_scope::table)
        .select((schema::api_keys::id, schema::api_keys::deleted, schema::api_scope::api_key))
        .filter(schema::api_keys::key_hash.eq(hash.clone()));
        let table_res = connection.interact(|conn|{
            table_res.get_result::<(i32, bool, String)>(conn)
            .optional()
        }).await
        .map_err(|e| panic!("Error Occurred in Database: {}", e))
        .unwrap()
        .map_err(|e| panic!("Error Occurred in Database async wrapper: {}",e))
        .unwrap();
        if let Some((_, true, _)) = table_res {
            println!("API Key was valid but is deleted. Hash: {}", hash);
            return None;
        }else if let Some((_, _,  scope)) = table_res {
            return Some(APIScope::try_from(scope.as_str()).unwrap());
        }else{
            return None;
        }
    }
}

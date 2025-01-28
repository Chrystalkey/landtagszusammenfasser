use crate::{error::LTZFError, LTZFServer, Result};
use diesel::prelude::*;
use crate::db::schema;
use axum::async_trait;
use openapi::apis::ApiKeyAuthHeader;
use sha256::digest;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

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
impl ToString for APIScope{
    fn to_string(&self) -> String {
        match self {
            APIScope::KeyAdder => "keyadder".to_string(),
            APIScope::Admin => "admin".to_string(),
            APIScope::Collector => "collector".to_string(),
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
        .select((schema::api_keys::id, schema::api_keys::deleted, schema::api_keys::expires_at, schema::api_scope::api_key))
        .filter(schema::api_keys::key_hash.eq(hash.clone()));
        let table_res = connection.interact(|conn|{
            table_res.get_result::<(i32, bool, chrono::NaiveDateTime, String)>(conn)
            .optional()
        }).await
        .map_err(|e| panic!("Error Occurred in Database: {}", e))
        .unwrap()
        .map_err(|e| panic!("Error Occurred in Database async wrapper: {}",e))
        .unwrap();

        match table_res{
            Some((_, true, _, _)) => {
                println!("API Key was valid but is deleted. Hash: {}", hash);
                return None;
            }
            Some((_, _, expires_at, scope)) => {
                if expires_at < chrono::Utc::now().naive_utc(){
                    println!("API Key was valid but is expired. Hash: {}", hash);
                    return None;
                }
                return Some(APIScope::try_from(scope.as_str()).unwrap());
            },
            None => {
                return None;
            }
        }
    }
}

pub async fn generate_api_key() -> String{
    let key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(59)
        .map(char::from)
        .chain("ltzf_".chars())
        .collect();
    key
}

pub async fn create_api_key(server: &LTZFServer, key: &str, scope: APIScope, created_by: &str) -> Result<i32> {
    let hash = digest(key);
    let connection = server.database.get().await?;
    let ret = connection.interact(move |conn|{
        conn.transaction(|conn|{
            diesel::insert_into(schema::api_keys::table)
            .values((
                schema::api_keys::key_hash.eq(hash),
                schema::api_keys::scope.eq(schema::api_scope::table.filter(schema::api_scope::api_key.eq(scope.to_string())).select(schema::api_scope::id).first::<i32>(conn)?),
                schema::api_keys::deleted.eq(false),
            ))
            .returning(schema::api_keys::id)
            .get_result::<i32>(conn)
        })
    }).await??;
    return Ok(ret)
}

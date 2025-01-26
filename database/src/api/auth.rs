use crate::{Result, LTZFServer};
use diesel::prelude::*;
use crate::db::schema;
use axum::async_trait;
use openapi::apis::ApiKeyAuthHeader;
use sha256::digest;
use std::{error::Error, hash::{DefaultHasher, Hash, Hasher}};

#[async_trait]
impl ApiKeyAuthHeader for LTZFServer{
    type Claims = Result<()>;
    async fn extract_claims_from_header(& self, headers: & axum::http::header::HeaderMap, key: & str) ->  Option<Self::Claims> {
        let hash = digest(key);

        let connection = self.database.get().await
        .map_err(|e| println!("An Error Occurred trying to get a database connection: {}", e));
        let table_res = schema::api_keys::table
        .select((schema::api_keys::id, schema::api_keys::coll_id, schema::api_keys::deleted))
        .filter(schema::api_keys::key_hash.eq(hash));
        let table_res = connection.interact(|conn|{
            table_res.get_result::<(i32, uuid::Uuid, bool)>(conn)
            .optional()
        }).await
        .map_err(|e| println!("Error Occurred in Database: {}", e))
        .unwrap()
        .map_err(|e| println!("Error Occurred in Database async wrapper: {}",e))
        .unwrap();
        if let Some((_, _, true)) = table_res {
            println!("API Key was valid but is deleted. Hash: {}", hash);
            return None;
        }else if let Some(x) = table_res{
            return Some(Ok())
        }else{
            return None;
        }
    }
}

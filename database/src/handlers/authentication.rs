// This module will at some point contain all logic responsible for authenticating collectors
use std::sync::Arc;
use axum::http::HeaderMap;
use uuid::Uuid;

use crate::AppState;
use crate::error::Result;

pub(crate) async fn authenticate_collector(
    _coll_id: Uuid,
    _headers: &HeaderMap, 
    _arc: Arc<AppState>) -> Result<()>{
    Ok(())
}
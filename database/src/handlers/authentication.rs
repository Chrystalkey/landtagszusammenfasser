// This module will at some point contain all logic responsible for authenticating collectors
use std::sync::Arc;
use std::collections::HashMap;
use axum::http::HeaderMap;

use crate::AppState;
use crate::error::Result;

pub(crate) async fn authenticate_collector(
    _headers: &HeaderMap, _arc: Arc<AppState>) -> Result<bool>{
    Ok(true)
}
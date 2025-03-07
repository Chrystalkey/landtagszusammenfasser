pub mod vorgang;
pub mod assitzung;
use  crate::{Result, LTZFServer};
use axum::async_trait;
use sqlx::PgConnection;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    NoMatch,
}
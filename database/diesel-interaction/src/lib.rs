use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DieselInteractionError{
    #[error("Diesel Error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Deadpool Error: {0}")]
    DeadpoolError(#[from] deadpool_diesel::InteractError),
}

#[derive(Debug, Serialize)]
pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
    /// 0-based index
    pub page: i64,
    pub page_size: i64,
    pub num_pages: i64,
}

pub type Result<T> = std::result::Result<T, DieselInteractionError>;
pub trait DieselInteraction<UpdateTable, ConnectionType, PaginationType>
where 
Self: Sized, 
{
    #[allow(async_fn_in_trait)]
    async fn create(conn: &mut ConnectionType, it: &Self) -> crate::Result<Self>;
    #[allow(async_fn_in_trait)]
    async fn update(conn: &mut ConnectionType, id: i32, ut: &UpdateTable) -> Result<Self>;

    #[allow(async_fn_in_trait)]
    async fn get(conn: &mut ConnectionType, id: i32) -> crate::Result<Self>;
    #[allow(async_fn_in_trait)]
    async fn matches(conn: &mut ConnectionType, ut: &UpdateTable) -> crate::Result<Vec<Self>>;
    
    #[allow(async_fn_in_trait)]
    async fn paginate(conn: &mut ConnectionType, page: i64, page_size: i64) -> crate::Result<PaginationType>;
    #[allow(async_fn_in_trait)]
    async fn delete(conn: &mut ConnectionType, id: i32) -> crate::Result<usize>;
}
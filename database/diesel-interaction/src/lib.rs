use serde::Serialize;
use diesel::*;

#[derive(Debug, Serialize)]
pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
    /// 0-based index
    pub page: i64,
    pub page_size: i64,
    pub num_pages: i64,
}

pub trait DieselInteraction<UpdateTable, ConnectionType, PaginationType>
where 
Self: Sized, 
{
    fn create(it: &Self, conn: &mut ConnectionType) -> QueryResult<Self>;
    fn update(conn: &mut ConnectionType, id: i32, ut: &UpdateTable) -> QueryResult<Self>;

    fn get(conn: &mut ConnectionType, id: i32) -> QueryResult<Self>;
    fn matches(conn: &mut ConnectionType, ut: &UpdateTable) -> QueryResult<Vec<Self>>;
    
    fn paginate(conn: &mut ConnectionType, page: i64, page_size: i64) -> QueryResult<PaginationType>;
    fn delete(conn: &mut ConnectionType, id: i32) -> QueryResult<usize>;
}
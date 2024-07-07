mod db;
mod gesetzesvorhaben;
use crate::error::DatabaseError;

pub trait EntityDBInteraction<T>
where
    Self: Sized + Sync + Send + Default,
    T: Sized + Copy + Sync + Send,
{
    async fn insert(&self, pool: &deadpool_diesel::postgres::Pool) -> Result<(), DatabaseError>;
    async fn update(&self, pool: &deadpool_diesel::postgres::Pool) -> Result<(), DatabaseError>;
    async fn delete(&self, pool: &deadpool_diesel::postgres::Pool) -> Result<(), DatabaseError>;
    async fn get_by_id(
        &self,
        id: T,
        pool: &deadpool_diesel::postgres::Pool,
    ) -> Result<Self, DatabaseError>;
    async fn get_all(
        &self,
        pool: &deadpool_diesel::postgres::Pool,
    ) -> Result<Vec<Self>, DatabaseError>;
}

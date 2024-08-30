/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::parlamente::Parlamente;

type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=status, primary_key(id), belongs_to(Parlamente, foreign_key=parlament))]
pub struct Statu {
    pub id: i32,
    pub name: String,
    pub parlament: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=status)]
pub struct CreateStatu {
    pub id: i32,
    pub name: String,
    pub parlament: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=status)]
pub struct UpdateStatu {
    pub name: Option<String>,
    pub parlament: Option<Option<i32>>,
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

impl Statu {

    pub fn create(db: &mut Connection, item: &CreateStatu) -> QueryResult<Self> {
        use crate::schema::status::dsl::*;

        insert_into(status).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::status::dsl::*;

        status.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::status::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = status.count().get_result(db)?;
        let items = status.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateStatu) -> QueryResult<Self> {
        use crate::schema::status::dsl::*;

        diesel::update(status.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::status::dsl::*;

        diesel::delete(status.filter(id.eq(param_id))).execute(db)
    }

}
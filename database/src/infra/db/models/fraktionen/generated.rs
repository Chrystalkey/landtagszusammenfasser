/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};


type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name=fraktionen, primary_key(id))]
pub struct Fraktionen {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=fraktionen)]
pub struct CreateFraktionen {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=fraktionen)]
pub struct UpdateFraktionen {
    pub name: Option<String>,
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

impl Fraktionen {

    pub fn create(db: &mut Connection, item: &CreateFraktionen) -> QueryResult<Self> {
        use crate::schema::fraktionen::dsl::*;

        insert_into(fraktionen).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::fraktionen::dsl::*;

        fraktionen.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::fraktionen::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = fraktionen.count().get_result(db)?;
        let items = fraktionen.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateFraktionen) -> QueryResult<Self> {
        use crate::schema::fraktionen::dsl::*;

        diesel::update(fraktionen.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::fraktionen::dsl::*;

        diesel::delete(fraktionen.filter(id.eq(param_id))).execute(db)
    }

}
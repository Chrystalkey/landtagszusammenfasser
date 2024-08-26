/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::dokumenttypen::Dokumenttypen;
use crate::models::gesetzesvorhaben::Gesetzesvorhaben;

type Connection = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=dokumente, primary_key(id), belongs_to(Dokumenttypen, foreign_key=typ) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct Dokumente {
    pub id: i32,
    pub off_id: String,
    pub datum: chrono::NaiveDate,
    pub url: String,
    pub collector_url: String,
    pub file: Option<String>,
    pub hash: String,
    pub gesetzesvorhaben: Option<i32>,
    pub typ: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=dokumente)]
pub struct CreateDokumente {
    pub id: i32,
    pub off_id: String,
    pub datum: chrono::NaiveDate,
    pub url: String,
    pub collector_url: String,
    pub file: Option<String>,
    pub hash: String,
    pub gesetzesvorhaben: Option<i32>,
    pub typ: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=dokumente)]
pub struct UpdateDokumente {
    pub off_id: Option<String>,
    pub datum: Option<chrono::NaiveDate>,
    pub url: Option<String>,
    pub collector_url: Option<String>,
    pub file: Option<Option<String>>,
    pub hash: Option<String>,
    pub gesetzesvorhaben: Option<Option<i32>>,
    pub typ: Option<Option<i32>>,
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

impl Dokumente {

    pub fn create(db: &mut Connection, item: &CreateDokumente) -> QueryResult<Self> {
        use crate::schema::dokumente::dsl::*;

        insert_into(dokumente).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::dokumente::dsl::*;

        dokumente.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::dokumente::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = dokumente.count().get_result(db)?;
        let items = dokumente.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateDokumente) -> QueryResult<Self> {
        use crate::schema::dokumente::dsl::*;

        diesel::update(dokumente.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::dokumente::dsl::*;

        diesel::delete(dokumente.filter(id.eq(param_id))).execute(db)
    }

}
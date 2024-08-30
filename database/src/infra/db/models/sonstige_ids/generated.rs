/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::gesetzesvorhaben::Gesetzesvorhaben;

type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=sonstige_ids, primary_key(id), belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct SonstigeId {
    pub id: i32,
    pub gesetzesvorhaben: Option<i32>,
    pub typ: String,
    pub inhalt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=sonstige_ids)]
pub struct CreateSonstigeId {
    pub id: i32,
    pub gesetzesvorhaben: Option<i32>,
    pub typ: String,
    pub inhalt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=sonstige_ids)]
pub struct UpdateSonstigeId {
    pub gesetzesvorhaben: Option<Option<i32>>,
    pub typ: Option<String>,
    pub inhalt: Option<String>,
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

impl SonstigeId {

    pub fn create(db: &mut Connection, item: &CreateSonstigeId) -> QueryResult<Self> {
        use crate::schema::sonstige_ids::dsl::*;

        insert_into(sonstige_ids).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::sonstige_ids::dsl::*;

        sonstige_ids.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::sonstige_ids::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = sonstige_ids.count().get_result(db)?;
        let items = sonstige_ids.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateSonstigeId) -> QueryResult<Self> {
        use crate::schema::sonstige_ids::dsl::*;

        diesel::update(sonstige_ids.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::sonstige_ids::dsl::*;

        diesel::delete(sonstige_ids.filter(id.eq(param_id))).execute(db)
    }

}
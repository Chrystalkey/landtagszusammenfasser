/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::abstimmungen::Abstimmungen;
use crate::models::dokumente::Dokumente;
use crate::models::tops::Top;

type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=tagesordnungspunkt, primary_key(id), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Dokumente, foreign_key=document) , belongs_to(Top, foreign_key=tops_id))]
pub struct Tagesordnungspunkt {
    pub id: i32,
    pub titel: String,
    pub tops_id: Option<i32>,
    pub document: Option<i32>,
    pub abstimmung: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=tagesordnungspunkt)]
pub struct CreateTagesordnungspunkt {
    pub id: i32,
    pub titel: String,
    pub tops_id: Option<i32>,
    pub document: Option<i32>,
    pub abstimmung: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=tagesordnungspunkt)]
pub struct UpdateTagesordnungspunkt {
    pub titel: Option<String>,
    pub tops_id: Option<Option<i32>>,
    pub document: Option<Option<i32>>,
    pub abstimmung: Option<Option<i32>>,
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

impl Tagesordnungspunkt {

    pub fn create(db: &mut Connection, item: &CreateTagesordnungspunkt) -> QueryResult<Self> {
        use crate::schema::tagesordnungspunkt::dsl::*;

        insert_into(tagesordnungspunkt).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::tagesordnungspunkt::dsl::*;

        tagesordnungspunkt.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::tagesordnungspunkt::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = tagesordnungspunkt.count().get_result(db)?;
        let items = tagesordnungspunkt.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateTagesordnungspunkt) -> QueryResult<Self> {
        use crate::schema::tagesordnungspunkt::dsl::*;

        diesel::update(tagesordnungspunkt.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::tagesordnungspunkt::dsl::*;

        diesel::delete(tagesordnungspunkt.filter(id.eq(param_id))).execute(db)
    }

}
/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::abstimmungen::Abstimmungen;
use crate::models::fraktionen::Fraktionen;

type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=abstimmungsergebnisse, primary_key(id), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Fraktionen, foreign_key=fraktion))]
pub struct Abstimmungsergebnisse {
    pub id: i32,
    pub abstimmung: Option<i32>,
    pub fraktion: Option<i32>,
    pub anteil: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=abstimmungsergebnisse)]
pub struct CreateAbstimmungsergebnisse {
    pub id: i32,
    pub abstimmung: Option<i32>,
    pub fraktion: Option<i32>,
    pub anteil: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=abstimmungsergebnisse)]
pub struct UpdateAbstimmungsergebnisse {
    pub abstimmung: Option<Option<i32>>,
    pub fraktion: Option<Option<i32>>,
    pub anteil: Option<f64>,
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

impl Abstimmungsergebnisse {

    pub fn create(db: &mut Connection, item: &CreateAbstimmungsergebnisse) -> QueryResult<Self> {
        use crate::schema::abstimmungsergebnisse::dsl::*;

        insert_into(abstimmungsergebnisse).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::abstimmungsergebnisse::dsl::*;

        abstimmungsergebnisse.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::abstimmungsergebnisse::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = abstimmungsergebnisse.count().get_result(db)?;
        let items = abstimmungsergebnisse.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateAbstimmungsergebnisse) -> QueryResult<Self> {
        use crate::schema::abstimmungsergebnisse::dsl::*;

        diesel::update(abstimmungsergebnisse.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::abstimmungsergebnisse::dsl::*;

        diesel::delete(abstimmungsergebnisse.filter(id.eq(param_id))).execute(db)
    }

}
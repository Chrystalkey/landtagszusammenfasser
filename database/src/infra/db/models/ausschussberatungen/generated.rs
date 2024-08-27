/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::ausschuesse::Ausschuesse;
use crate::models::dokumente::Dokumente;
use crate::models::gesetzesvorhaben::Gesetzesvorhaben;

type Connection = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=ausschussberatungen, primary_key(id), belongs_to(Ausschuesse, foreign_key=ausschuss) , belongs_to(Dokumente, foreign_key=dokument) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct Ausschussberatungen {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub datum: chrono::NaiveDate,
    pub gesetzesvorhaben: Option<i32>,
    pub ausschuss: Option<i32>,
    pub dokument: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=ausschussberatungen)]
pub struct CreateAusschussberatungen {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub datum: chrono::NaiveDate,
    pub gesetzesvorhaben: Option<i32>,
    pub ausschuss: Option<i32>,
    pub dokument: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=ausschussberatungen)]
pub struct UpdateAusschussberatungen {
    pub ext_id: Option<uuid::Uuid>,
    pub datum: Option<chrono::NaiveDate>,
    pub gesetzesvorhaben: Option<Option<i32>>,
    pub ausschuss: Option<Option<i32>>,
    pub dokument: Option<Option<i32>>,
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

impl Ausschussberatungen {

    pub fn create(db: &mut Connection, item: &CreateAusschussberatungen) -> QueryResult<Self> {
        use crate::schema::ausschussberatungen::dsl::*;

        insert_into(ausschussberatungen).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::ausschussberatungen::dsl::*;

        ausschussberatungen.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::ausschussberatungen::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = ausschussberatungen.count().get_result(db)?;
        let items = ausschussberatungen.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateAusschussberatungen) -> QueryResult<Self> {
        use crate::schema::ausschussberatungen::dsl::*;

        diesel::update(ausschussberatungen.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::ausschussberatungen::dsl::*;

        diesel::delete(ausschussberatungen.filter(id.eq(param_id))).execute(db)
    }

}
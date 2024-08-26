/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::abstimmungen::Abstimmungen;
use crate::models::gesetzesvorhaben::Gesetzesvorhaben;
use crate::models::status::Statu;

type Connection = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_status, primary_key(gesetzesvorhaben,status,abstimmung), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Statu, foreign_key=status))]
pub struct RelGesStatu {
    pub gesetzesvorhaben: i32,
    pub status: i32,
    pub abstimmung: i32,
    pub datum: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=rel_ges_status)]
pub struct CreateRelGesStatu {
    pub gesetzesvorhaben: i32,
    pub status: i32,
    pub abstimmung: i32,
    pub datum: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=rel_ges_status)]
pub struct UpdateRelGesStatu {
    pub datum: Option<chrono::NaiveDateTime>,
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

impl RelGesStatu {

    pub fn create(db: &mut Connection, item: &CreateRelGesStatu) -> QueryResult<Self> {
        use crate::schema::rel_ges_status::dsl::*;

        insert_into(rel_ges_status).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_gesetzesvorhaben: i32, param_status: i32, param_abstimmung: i32) -> QueryResult<Self> {
        use crate::schema::rel_ges_status::dsl::*;

        rel_ges_status.filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(status.eq(param_status)).filter(abstimmung.eq(param_abstimmung)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::rel_ges_status::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = rel_ges_status.count().get_result(db)?;
        let items = rel_ges_status.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_gesetzesvorhaben: i32, param_status: i32, param_abstimmung: i32, item: &UpdateRelGesStatu) -> QueryResult<Self> {
        use crate::schema::rel_ges_status::dsl::*;

        diesel::update(rel_ges_status.filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(status.eq(param_status)).filter(abstimmung.eq(param_abstimmung))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_gesetzesvorhaben: i32, param_status: i32, param_abstimmung: i32) -> QueryResult<usize> {
        use crate::schema::rel_ges_status::dsl::*;

        diesel::delete(rel_ges_status.filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(status.eq(param_status)).filter(abstimmung.eq(param_abstimmung))).execute(db)
    }

}
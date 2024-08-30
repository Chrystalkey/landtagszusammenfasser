/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::abstimmungen::Abstimmungen;
use crate::models::dokumente::Dokumente;
use crate::models::gesetzesvorhaben::Gesetzesvorhaben;
use crate::models::tops::Top;

type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_tops, primary_key(top,gesetzesvorhaben,dokument,abstimmung), belongs_to(Abstimmungen, foreign_key=abstimmung) , belongs_to(Dokumente, foreign_key=dokument) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Top, foreign_key=top))]
pub struct RelGesTop {
    pub top: i32,
    pub gesetzesvorhaben: i32,
    pub abstimmung: i32,
    pub dokument: i32,
    pub titel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=rel_ges_tops)]
pub struct CreateRelGesTop {
    pub top: i32,
    pub gesetzesvorhaben: i32,
    pub abstimmung: i32,
    pub dokument: i32,
    pub titel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=rel_ges_tops)]
pub struct UpdateRelGesTop {
    pub titel: Option<String>,
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

impl RelGesTop {

    pub fn create(db: &mut Connection, item: &CreateRelGesTop) -> QueryResult<Self> {
        use crate::schema::rel_ges_tops::dsl::*;

        insert_into(rel_ges_tops).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_top: i32, param_gesetzesvorhaben: i32, param_dokument: i32, param_abstimmung: i32) -> QueryResult<Self> {
        use crate::schema::rel_ges_tops::dsl::*;

        rel_ges_tops.filter(top.eq(param_top)).filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(dokument.eq(param_dokument)).filter(abstimmung.eq(param_abstimmung)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::rel_ges_tops::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = rel_ges_tops.count().get_result(db)?;
        let items = rel_ges_tops.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_top: i32, param_gesetzesvorhaben: i32, param_dokument: i32, param_abstimmung: i32, item: &UpdateRelGesTop) -> QueryResult<Self> {
        use crate::schema::rel_ges_tops::dsl::*;

        diesel::update(rel_ges_tops.filter(top.eq(param_top)).filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(dokument.eq(param_dokument)).filter(abstimmung.eq(param_abstimmung))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_top: i32, param_gesetzesvorhaben: i32, param_dokument: i32, param_abstimmung: i32) -> QueryResult<usize> {
        use crate::schema::rel_ges_tops::dsl::*;

        diesel::delete(rel_ges_tops.filter(top.eq(param_top)).filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(dokument.eq(param_dokument)).filter(abstimmung.eq(param_abstimmung))).execute(db)
    }

}
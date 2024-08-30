/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::gesetzesvorhaben::Gesetzesvorhaben;
use crate::models::schlagworte::Schlagworte;

type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_schlagworte, primary_key(gesetzesvorhaben,schlagwort), belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben) , belongs_to(Schlagworte, foreign_key=schlagwort))]
pub struct RelGesSchlagworte {
    pub gesetzesvorhaben: i32,
    pub schlagwort: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable)]
#[diesel(table_name=rel_ges_schlagworte)]
pub struct CreateRelGesSchlagworte {
    pub gesetzesvorhaben: i32,
    pub schlagwort: i32,
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

impl RelGesSchlagworte {

    pub fn create(db: &mut Connection, item: &CreateRelGesSchlagworte) -> QueryResult<Self> {
        use crate::schema::rel_ges_schlagworte::dsl::*;

        insert_into(rel_ges_schlagworte).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_gesetzesvorhaben: i32, param_schlagwort: i32) -> QueryResult<Self> {
        use crate::schema::rel_ges_schlagworte::dsl::*;

        rel_ges_schlagworte.filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(schlagwort.eq(param_schlagwort)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::rel_ges_schlagworte::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = rel_ges_schlagworte.count().get_result(db)?;
        let items = rel_ges_schlagworte.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn delete(db: &mut Connection, param_gesetzesvorhaben: i32, param_schlagwort: i32) -> QueryResult<usize> {
        use crate::schema::rel_ges_schlagworte::dsl::*;

        diesel::delete(rel_ges_schlagworte.filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(schlagwort.eq(param_schlagwort))).execute(db)
    }

}
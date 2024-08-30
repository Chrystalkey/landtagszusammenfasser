/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::gesetzeseigenschaften::Gesetzeseigenschaften;
use crate::models::gesetzesvorhaben::Gesetzesvorhaben;

type Connection = diesel::pg::PgConnection;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, Selectable)]
#[diesel(table_name=rel_ges_eigenschaft, primary_key(gesetzesvorhaben,eigenschaft), belongs_to(Gesetzeseigenschaften, foreign_key=eigenschaft) , belongs_to(Gesetzesvorhaben, foreign_key=gesetzesvorhaben))]
pub struct RelGesEigenschaft {
    pub gesetzesvorhaben: i32,
    pub eigenschaft: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable)]
#[diesel(table_name=rel_ges_eigenschaft)]
pub struct CreateRelGesEigenschaft {
    pub gesetzesvorhaben: i32,
    pub eigenschaft: i32,
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

impl RelGesEigenschaft {

    pub fn create(db: &mut Connection, item: &CreateRelGesEigenschaft) -> QueryResult<Self> {
        use crate::schema::rel_ges_eigenschaft::dsl::*;

        insert_into(rel_ges_eigenschaft).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_gesetzesvorhaben: i32, param_eigenschaft: i32) -> QueryResult<Self> {
        use crate::schema::rel_ges_eigenschaft::dsl::*;

        rel_ges_eigenschaft.filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(eigenschaft.eq(param_eigenschaft)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::rel_ges_eigenschaft::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = rel_ges_eigenschaft.count().get_result(db)?;
        let items = rel_ges_eigenschaft.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn delete(db: &mut Connection, param_gesetzesvorhaben: i32, param_eigenschaft: i32) -> QueryResult<usize> {
        use crate::schema::rel_ges_eigenschaft::dsl::*;

        diesel::delete(rel_ges_eigenschaft.filter(gesetzesvorhaben.eq(param_gesetzesvorhaben)).filter(eigenschaft.eq(param_eigenschaft))).execute(db)
    }

}
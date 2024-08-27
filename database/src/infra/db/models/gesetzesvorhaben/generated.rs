/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use crate::models::ausschuesse::Ausschuesse;
use crate::models::initiatoren::Initiatoren;

type Connection = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=gesetzesvorhaben, primary_key(id), belongs_to(Ausschuesse, foreign_key=feder) , belongs_to(Initiatoren, foreign_key=initiat))]
pub struct Gesetzesvorhaben {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub titel: String,
    pub off_titel: String,
    pub url_gesblatt: Option<String>,
    pub id_gesblatt: Option<String>,
    pub verfassungsaendernd: bool,
    pub trojaner: Option<bool>,
    pub feder: Option<i32>,
    pub initiat: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=gesetzesvorhaben)]
pub struct CreateGesetzesvorhaben {
    pub id: i32,
    pub ext_id: uuid::Uuid,
    pub titel: String,
    pub off_titel: String,
    pub url_gesblatt: Option<String>,
    pub id_gesblatt: Option<String>,
    pub verfassungsaendernd: bool,
    pub trojaner: Option<bool>,
    pub feder: Option<i32>,
    pub initiat: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=gesetzesvorhaben)]
pub struct UpdateGesetzesvorhaben {
    pub ext_id: Option<uuid::Uuid>,
    pub titel: Option<String>,
    pub off_titel: Option<String>,
    pub url_gesblatt: Option<Option<String>>,
    pub id_gesblatt: Option<Option<String>>,
    pub verfassungsaendernd: Option<bool>,
    pub trojaner: Option<Option<bool>>,
    pub feder: Option<Option<i32>>,
    pub initiat: Option<Option<i32>>,
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

impl Gesetzesvorhaben {

    pub fn create(db: &mut Connection, item: &CreateGesetzesvorhaben) -> QueryResult<Self> {
        use crate::schema::gesetzesvorhaben::dsl::*;

        insert_into(gesetzesvorhaben).values(item).get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::gesetzesvorhaben::dsl::*;

        gesetzesvorhaben.filter(id.eq(param_id)).first::<Self>(db)
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::gesetzesvorhaben::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = gesetzesvorhaben.count().get_result(db)?;
        let items = gesetzesvorhaben.limit(page_size).offset(page * page_size).load::<Self>(db)?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub fn update(db: &mut Connection, param_id: i32, item: &UpdateGesetzesvorhaben) -> QueryResult<Self> {
        use crate::schema::gesetzesvorhaben::dsl::*;

        diesel::update(gesetzesvorhaben.filter(id.eq(param_id))).set(item).get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::gesetzesvorhaben::dsl::*;

        diesel::delete(gesetzesvorhaben.filter(id.eq(param_id))).execute(db)
    }

}
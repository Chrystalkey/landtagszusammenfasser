extern crate diesel_interaction;

use diesel_interaction::*;
use diesel_interaction_derive::DieselInteraction;
use diesel::*;
use super::schema::*;
use serde::{Deserialize, Serialize};

type Connection = PgConnection;

#[derive(DieselInteraction, Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[schema_table = "dokumenttypen"]
#[diesel(table_name=dokumenttypen, primary_key(id))]
pub struct Dokumenttypen {
    pub id: i32,
    pub name: String,
}




// #[derive(Debug, Serialize)]
// pub struct PaginationResult<T> {
//     pub items: Vec<T>,
//     pub total_items: i64,
//     /// 0-based index
//     pub page: i64,
//     pub page_size: i64,
//     pub num_pages: i64,
// }


// macro_rules! db_interactions{
//     ($tablestruct:ident, $schema_table:ident, $update_table_type:ident) =>{
//         impl DBInteraction<$update_table_type, 
//         Connection, 
//         PaginationResult<Self>> for $tablestruct {
//             fn create(it: &Self, conn: &mut Connection) -> QueryResult<Self> {
//                 use crate::schema::$schema_table::dsl::*;
//                 insert_into($schema_table).values(it).get_result::<Self>(conn)
//             }
//             fn update(conn: &mut Connection, id: i32, ut: &$update_table_type) -> QueryResult<Self> {
//                 use crate::schema::$schema_table::dsl::*;
//                 diesel::update($schema_table.filter(id.eq(id))).set(ut).get_result(conn)
//             }
//             fn get(conn: &mut Connection, id: i32) -> QueryResult<Self> {
//                 use crate::schema::$schema_table::dsl::*;
//                 $schema_table.filter(id.eq(id)).first::<Self>(conn)
//             }
//             fn matches(conn: &mut Connection, ut: &$update_table_type) -> QueryResult<Self> {
//                 todo!()
//             }
//             fn paginate(conn: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
//                 use crate::schema::$schema_table::dsl::*;
//                 let page_size = if page_size < 1 { 1 } else { page_size };
//                 let total_items = $schema_table.count().get_result(conn)?;
//                 let items = $schema_table.limit(page_size).offset(page * page_size).load::<Self>(conn)?;
//                 Ok(PaginationResult {
//                     items,
//                     total_items,
//                     page,
//                     page_size,
//                     num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
//                 })
//             }
//             fn delete(conn: &mut Connection, id: i32) -> QueryResult<usize> {
//                 use crate::schema::$schema_table::dsl::*;
//                 diesel::delete($schema_table.filter(id.eq(id))).execute(conn)
//             }
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
// #[diesel(table_name=dokumenttypen)]
// pub struct UpdateDokumenttypen {
//     pub name: Option<String>,
// }
// impl DBInteraction<UpdateDokumenttypen, 
//     Connection, 
//     PaginationResult<Self>> for Dokumenttypen {
//         fn create(it: &Self, conn: &mut Connection) -> QueryResult<Self> {
//             use crate::schema::dokumenttypen::dsl::*;
//             insert_into(dokumenttypen).values(it).get_result::<Self>(conn)
//         }
//         fn update(conn: &mut Connection, id: i32, ut: &UpdateDokumenttypen) -> QueryResult<Self> {
//             use crate::schema::dokumenttypen::dsl::*;
//             diesel::update(dokumenttypen.filter(id.eq(id))).set(ut).get_result(conn)
//         }
//         fn get(conn: &mut Connection, id: i32) -> QueryResult<Self> {
//             use crate::schema::dokumenttypen::dsl::*;
//             dokumenttypen.filter(id.eq(id)).first::<Self>(conn)
//         }
//         fn matches(conn: &mut Connection, ut: &UpdateDokumenttypen) -> QueryResult<Vec<Self>> {
//             use crate::schema::dokumenttypen::dsl as table;
//             let mut query = table::dokumenttypen.into_boxed();
//             if let Some(ut_val) = &ut.name {
//                 query = query.filter(table::name.eq(ut_val));
//             }
//             query.load::<Self>(conn)
//         }
//         fn paginate(conn: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
//             use crate::schema::dokumenttypen::dsl::*;
//             let page_size = if page_size < 1 { 1 } else { page_size };
//             let total_items = dokumenttypen.count().get_result(conn)?;
//             let items = dokumenttypen.limit(page_size).offset(page * page_size).load::<Self>(conn)?;
//             Ok(PaginationResult {
//                 items,
//                 total_items,
//                 page,
//                 page_size,
//                 num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
//             })
//         }
//         fn delete(conn: &mut Connection, id: i32) -> QueryResult<usize> {
//             use crate::schema::dokumenttypen::dsl::*;
//             diesel::delete(dokumenttypen.filter(id.eq(id))).execute(conn)
//         }
//     }
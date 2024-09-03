use serde::Serialize;
use thiserror::Error;

///
/// The Following Structure:
/// ```
/// #[derive(DieselInteraction)]
/// #[connection_type = deadpool_diesel::postgres::Connection]
/// #[diesel(table_name=crate::schema::parlamente)]
/// #[diesel(primary_key(id))]
/// #[diesel(check_for_backend(diesel::pg::Pg))]
/// struct Parlamente{
///   id: i32,
///   name: String,
///   kurzname: String,
/// }
/// ```
/// Notes: 
/// - It is required that the table_name is a separate attribute and the whole primary_key/belongs_to thing is one. This is due to simplified parser logic because I was a bit lazy.
/// - Debug and Clone are derived for all structures
/// - it only supports structs with a singular primary key called id with type i32
///
/// This will result in the following code being generated:
/// ```
/// mod parlamente{
///     use crate::schema::parlamente::dsl as module;
///     use crate::schema::parlamente::table;
///     use diesel::*;
///     use diesel_interaction::{DieselInteractionError, PaginationResult};
///     type Connection = deadpool_diesel::postgre::Connection;
///     type Result<T> = std::result::Result<T, DieselInteractionError>;
/// 
///     #[derive(Debug, Clone, Selectable, Queryable, Identifiable)]
///     #[diesel(table_name=crate::schema::parlamente, primary_key(id))]
///     #[diesel(check_for_backend(diesel::pg::Pg))]
///     pub struct Row {
///         pub id: i32,
///         pub name: String,
///         pub kurzname: String,
///     }
///     #[derive(Debug, Clone, AsChangeset)]
///     #[diesel(table_name=parlamente)]
///     #[diesel(check_for_backend(diesel::pg::Pg))]
///     pub struct Update {
///         pub name: Option<String>,
///         pub kurzname: Option<String>,
///     }
///     #[derive(Debug, Clone, Insertable)]
///     #[diesel(table_name=parlamente)]
///     #[diesel(check_for_backend(diesel::pg::Pg))]
///     pub struct Insert {
///         pub name: String,
///         pub kurzname: String,
///     }
///     impl From<Row> for Insert {
///         fn from(row: Row) -> Self {
///             Self {
///                 name: row.name,
///                 kurzname: row.kurzname,
///             }
///         }
///     }
///     impl From<Row> for Update {
///         fn from(row: Row) -> Self {
///             Self {
///                 name: Some(row.name),
///                 kurzname: Some(row.kurzname),
///             }
///         }
///     }
///     async fn insert(conn: &mut Connection, it: Insert) -> Result<usize> {
///         let result = conn
///             .interact(move |conn| diesel::insert_into(table).values(&it).execute(conn))
///             .await??;
///         Ok(result)
///     }
///     async fn update(conn: &mut Connection, id: i32, ut: &Update) -> Result<usize> {
///         let utcl = ut.clone();
///         let result = conn
///             .interact(move |conn| {
///                 diesel::update(table.filter(module::id.eq(id)))
///                     .set(utcl)
///                     .execute(conn)
///             })
///             .await??;
///         Ok(result)
///     }
///     async fn select(conn: &mut Connection, id: i32) -> Result<Row> {
///         let result = conn
///             .interact(move |conn| {
///                 table
///                     .filter(module::id.eq(id))
///                     .select(Row::as_select())
///                     .get_result(conn)
///             })
///             .await??;
///         Ok(result)
///     }
///     async fn select_matching(conn: &mut Connection, ut: Update) -> Result<Vec<Row>> {
///         let result = conn
///             .interact(move |conn| {
///                 let mut query = table.into_boxed();
///                 if let Some(ut_val) = &ut.name {
///                     query = query.filter(module::name.eq(ut_val));
///                 }
///                 if let Some(ut_val) = &ut.kurzname {
///                     query = query.filter(module::kurzname.eq(ut_val));
///                 }
///                 query.load::<Row>(conn)
///             })
///             .await??;
///         Ok(result)
///     }
///     async fn paginate(
///         conn: &mut Connection,
///         page: i64,
///         page_size: i64,
///     ) -> Result<PaginationResult<Row>> {
///         let page_size = if page_size < 1 { 1 } else { page_size };
///         let total_items = conn
///             .interact(|conn| table.count().get_result(conn))
///             .await??;
///         let items = conn
///             .interact(move |conn| {
///                 table
///                     .limit(page_size)
///                     .offset(page * page_size)
///                     .load::<Row>(conn)
///             })
///             .await??;
///         Ok(PaginationResult {
///             items,
///             total_items,
///             page,
///             page_size,
///             num_pages: total_items / page_size + i64::from(total_items % page_size != 0),
///         })
///     }
///     async fn delete(conn: &mut Connection, id: i32) -> Result<usize> {
///         let result = conn
///             .interact(move |conn| diesel::delete(table.filter(module::id.eq(id))).execute(conn))
///             .await??;
///         Ok(result)
///     }
/// }
/// ```

#[derive(Error, Debug)]
pub enum DieselInteractionError {
    #[error("Diesel Error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Deadpool Error: {0}")]
    DeadpoolError(#[from] deadpool_diesel::InteractError),
}

#[derive(Debug, Serialize)]
pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
    //  0-based index
    pub page: i64,
    pub page_size: i64,
    pub num_pages: i64,
}

pub(crate) mod db;
pub mod api;

pub use db::schema;

pub enum DBClass{
    AutomaticallyFillable,
    ManuallyFillable
}
trait DatabaseClass{
    fn get_class(&self) -> DBClass{
        DBClass::ManuallyFillable
    }
}
macro_rules! impl_database_class {
    ($table:ident, $class:expr) => {
            impl DatabaseClass for schema::$table::table {
                fn get_class(&self) -> DBClass{
                    $class
                }
            }
    };
}

impl_database_class!(gesetzesvorhaben, DBClass::AutomaticallyFillable);
impl_database_class!(dokumente, DBClass::AutomaticallyFillable);
impl_database_class!(tops, DBClass::AutomaticallyFillable);
impl_database_class!(ausschussberatungen, DBClass::AutomaticallyFillable);
impl_database_class!(sonstige_ids, DBClass::AutomaticallyFillable);
impl_database_class!(abstimmungen, DBClass::AutomaticallyFillable);
impl_database_class!(tagesordnungspunkt, DBClass::AutomaticallyFillable);
impl_database_class!(abstimmungsergebnisse, DBClass::AutomaticallyFillable);
impl_database_class!(rel_ges_schlagworte, DBClass::AutomaticallyFillable);
impl_database_class!(rel_ges_eigenschaft, DBClass::AutomaticallyFillable);
impl_database_class!(rel_ges_status, DBClass::AutomaticallyFillable);
impl_database_class!(rel_ges_tops, DBClass::AutomaticallyFillable);
impl_database_class!(schlagworte, DBClass::AutomaticallyFillable);

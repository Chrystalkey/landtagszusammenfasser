pub mod schema;
pub mod connection;

#[macro_export]
macro_rules! required_field {
    ($value:expr) => {
        $value.clone().map_or(
            Err(DatabaseError::MissingFieldForInsert(format!(
                "{} is a required field",
                stringify!($value)
            ))),
            |x| Ok(x),
        )?
    };
}

#[macro_export]
macro_rules! async_db {
    ($conn:ident, $load_function:ident, $query:block) => {
        $conn
            .interact(move |c| $query.$load_function(c))
            .await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
            .map_err(DatabaseError::from)?
    };
}
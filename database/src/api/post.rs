use crate::Result;

/// Inserts a new GSVH into the database.
async fn insert_gsvh() -> Result<()> {todo!()}

/// Returns a list of all GSVHs with which it might be mergeable. 
/// If none are found, returns none.
async fn find_matches()->Result<Option<Vec<i32>>> {todo!()}

/// Merges two GSVHs into one, updating stations and data points as it goes along
async fn merge_gsvh(one: i32, two: i32) -> Result<()> {todo!()}
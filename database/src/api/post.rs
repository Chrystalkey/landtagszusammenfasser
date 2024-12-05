use crate::{Result, LTZFServer, db};
use diesel::Connection;
use openapi::models;

pub async fn api_v1_gesetzesvorhaben_post(
    server: &LTZFServer,
    api_gsvh: models::Gesetzesvorhaben
)-> Result<()> {
    let conn = server.database.get().await?;
    let res = conn.interact(
        move |conn| 
            conn.transaction(
                |conn| db::insert::insert_gsvh(&api_gsvh, conn)
            )
    ).await??;
    Ok(())
}

/// Returns a list of all GSVHs with which it might be mergeable. 
/// If none are found, returns none.
async fn find_matches(api_gsvh: & models::Gesetzesvorhaben)->Result<Option<Vec<i32>>> {todo!()}

/// Merges two GSVHs into one, updating stations and data points as it goes along
async fn merge_gsvh(one: i32, two: i32) -> Result<()> {todo!()}
use crate::Result;
use diesel::prelude::*;
use openapi::models;
use uuid::Uuid;
use crate::db::schema;

/// Inserts a new GSVH into the database.
fn insert_gsvh(
    api_gsvh: &models::Gesetzesvorhaben,
    connection: &mut diesel::PgConnection
) -> Result<i32> {
    use schema::gesetzesvorhaben::dsl;
    use schema::gesetzestyp::dsl as typ_dsl;
    let gsvh_id = 
    diesel::insert_into(schema::gesetzesvorhaben::table)
    .values(
        (
            dsl::api_id.eq(api_gsvh.api_id),
            dsl::titel.eq(&api_gsvh.titel),
            dsl::verfassungsaendernd.eq(api_gsvh.verfassungsaendernd),
            dsl::typ.eq(
                typ_dsl::gesetzestyp
                .select(typ_dsl::id)
                .filter(typ_dsl::api_key.eq(&api_gsvh.typ.to_string()))
                .first::<i32>(connection)?
            ),
        )
    )
    .returning(dsl::id)
    .get_result::<i32>(connection)?;
    // insert links, initiatoren, ids
    if let Some(links) = &api_gsvh.links {
        use schema::rel_gesvh_links::dsl as dsl;
        diesel::insert_into(schema::rel_gesvh_links::table)
        .values(
            links.iter()
            .cloned()
            .map(|s|
                dsl::link.eq(s))
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }

    if !api_gsvh.initiatoren.is_empty() {
        use schema::rel_gesvh_init::dsl as dsl;
        diesel::insert_into(schema::rel_gesvh_init::table)
        .values(
            api_gsvh.initiatoren.iter()
            .map(|s|
                dsl::initiator.eq(s))
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }

    if let Some(ids) = api_gsvh.ids.as_ref() {
        use schema::rel_gesvh_id::dsl as dsl;
        let mut value_vec = vec![];

        for id_entry in ids.iter(){
            let value= (
                dsl::gesetzesvorhaben_id.eq(gsvh_id),
                dsl::identifikator.eq(&id_entry.id),
                dsl::id_typ.eq(
                    schema::identifikatortyp::table
                    .select(schema::identifikatortyp::id)
                    .filter(schema::identifikatortyp::api_key
                        .eq(&id_entry.typ.to_string())
                    )
                    .first::<i32>(connection)?
                )
            );
            value_vec.push(value);
        }
        diesel::insert_into(schema::rel_gesvh_id::table)
        .values(&value_vec)
        .execute(connection)?;
    }
    
    if !api_gsvh.stationen.is_empty() {

    }

    Ok(gsvh_id)
}

/// Returns a list of all GSVHs with which it might be mergeable. 
/// If none are found, returns none.
async fn find_matches(api_gsvh: & models::Gesetzesvorhaben)->Result<Option<Vec<i32>>> {todo!()}

/// Merges two GSVHs into one, updating stations and data points as it goes along
async fn merge_gsvh(one: i32, two: i32) -> Result<()> {todo!()}
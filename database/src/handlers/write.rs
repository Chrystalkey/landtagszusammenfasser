use crate::infra::api::IdentifikatorTyp;
use diesel::prelude::*;
use std::sync::Arc;

extern crate diesel_interaction;
use crate::error::{DatabaseError, LTZFError};
use crate::infra::api;
use crate::infra::db::connection as dbcon;
use crate::infra::db::schema as dbschema;
use crate::{async_db, AppState};
use axum::http::StatusCode;

/// A gesvh is mergeable if:
/// new.titel == old.titel || new.ids(vorgang_id) == old.ids(vorgang_id)
async fn merge_candidates(
    gesvh: &api::Gesetzesvorhaben,
    conn: &mut deadpool_diesel::postgres::Connection,
) -> Result<Vec<i32>, DatabaseError> {
    let titel = gesvh.titel.clone();
    
    // definitely mergeable if the title is the same (and the api_id is different)
    let mut eligible_gesvh: Vec<i32> = async_db!(
        conn, load, {
            dbschema::gesetzesvorhaben::table
                .filter(dbschema::gesetzesvorhaben::titel.eq(titel))
                .select(dbschema::gesetzesvorhaben::id)
        }
    );
    tracing::trace!("Found elements {:?} with a matching title", eligible_gesvh);

    let id_elements: Vec<api::Identifikator> = gesvh
        .ids
        .iter()
        .filter(|&el| el.typ == IdentifikatorTyp::Vorgangsnummer)
        .cloned()
        .collect();

    if id_elements.len() > 0 {
        tracing::trace!("Checking for matching Vorgangsnummer");
        let result: Vec<i32> = async_db!(conn, load, {
            dbschema::gesetzesvorhaben::table
                    .inner_join(
                        dbschema::rel_gesvh_id::table.inner_join(dbschema::identifikatortyp::table)
                    )
                    .filter(
                        dbschema::rel_gesvh_id::identifikator
                            .eq(&id_elements[0].id)
                            .and(dbschema::identifikatortyp::value.eq("Vorgangsnummer")),
                    )
                    .select(dbschema::gesetzesvorhaben::id)
        });
        eligible_gesvh.extend(result);
    }
    tracing::trace!("Found elements {:?} with a matching Vorgangsnummer or matching title", eligible_gesvh);
    if eligible_gesvh.is_empty(){
        return Ok(eligible_gesvh);
    }
    let api_id = gesvh.api_id;
    let api_id_filtered: Vec<i32> = async_db!(
        conn, load, {
            let mut query = dbschema::gesetzesvorhaben::table.into_boxed();
            for id in &eligible_gesvh{
                query = query.or_filter(dbschema::gesetzesvorhaben::id.eq(*id)
                .and(dbschema::gesetzesvorhaben::api_id.ne(api_id)));
            }
            query.select(dbschema::gesetzesvorhaben::id)
        }
    );
    tracing::trace!("Found elements {:?} after filtering for equal api_ids", api_id_filtered);
    if api_id_filtered.is_empty() {
        tracing::warn!("No merge candidates found for Gesvh with api_id {} apart from the same entry", gesvh.api_id);
        return Err(DatabaseError::ApiIDEqual(gesvh.api_id));
    }
    Ok(api_id_filtered)
}

fn merge_gesvh(
    _gesvh: api::Gesetzesvorhaben,
    _conn: &mut diesel::pg::PgConnection,
) -> Result<(), DatabaseError> {
    todo!("Not yet implemented!")
}

fn helper_create_dokument(
    dok: api::Dokument,
    conn: &mut diesel::pg::PgConnection
) -> Result<i32, DatabaseError>{
    let dok_id = diesel::insert_into(dbschema::dokument::table)
            .values(&dbcon::dokument::Insert{
                dokumenttyp_id: dbschema::dokumenttyp::table
                .filter(dbschema::dokumenttyp::value.eq(format!("{:?}", dok.typ)))
                .select(dbschema::dokumenttyp::id)
                .first(conn)?,
                hash: dok.hash,
                titel: dok.titel,
                url: dok.url,
                zusammenfassung: dok.zusammenfassung,
            })
            .returning(dbschema::dokument::id)
            .get_result::<i32>(conn)?;
    let mut autor_ids = vec![];
    for aut in dok.autoren{
        let id_opt: Option<i32> = dbschema::autor::table
        .filter(dbschema::autor::name.eq(aut.name.clone()))
        .filter(dbschema::autor::organisation.eq(aut.organisation.clone()))
        .select(dbschema::autor::id)
        .first(conn)
        .optional()?;
        if let Some(id) = id_opt{
            autor_ids.push(id);
        }else{
            let id = diesel::insert_into(dbschema::autor::table)
            .values(
                (
                    dbschema::autor::name.eq(aut.name.clone()),
                    dbschema::autor::organisation.eq(aut.organisation.clone())
                )
            )
            .returning(dbschema::autor::id)
            .get_result(conn)?;
            autor_ids.push(id);
        }
    }
    diesel::insert_into(dbschema::rel_dok_autor::table)
    .values(autor_ids.iter().map(|id| (
        dbschema::rel_dok_autor::autor_id.eq(id),
        dbschema::rel_dok_autor::dokument_id.eq(dok_id)
    )).collect::<Vec<_>>()
    )
    .execute(conn)?;
    let sw_ids = helper_ins_or_ret_schlagwort(dok.schlagworte, conn)?;
    diesel::insert_into(dbschema::rel_dok_schlagwort::table)
    .values(
        sw_ids.iter().map(|id|(
            dbschema::rel_dok_schlagwort::dokument_id.eq(dok_id),
            dbschema::rel_dok_schlagwort::schlagwort_id.eq(id)
        )).collect::<Vec<_>>()
    )
    .execute(conn)?;
    Ok(dok_id)
}


/// returns ids of the inserted schlagworte
fn helper_ins_or_ret_schlagwort(
    schlagworte: Vec<String>,
    conn: &mut diesel::pg::PgConnection
) -> Result<Vec<i32>, DatabaseError>{
    let mut result_vec = vec![];
    for schlagwort in schlagworte{
        let db_return = dbschema::schlagwort::table.filter(dbschema::schlagwort::value.eq(schlagwort.to_lowercase()))
        .select(dbschema::schlagwort::id)
        .first::<i32>(conn)
        .optional()?;
        if let Some(id) = db_return {
            result_vec.push(id)
        }else{
            let id = diesel::insert_into(dbschema::schlagwort::table)
            .values(dbschema::schlagwort::value.eq(schlagwort.clone()))
            .returning(dbschema::schlagwort::id)
            .get_result(conn)?;
            result_vec.push(id);
        }
    }
    Ok(result_vec)
}

fn create_gesvh(
    gesvh: api::Gesetzesvorhaben,
    conn: &mut diesel::pg::PgConnection,
) -> Result<i32, DatabaseError> {
    // insert the gesvh itself
    let typ_variant = gesvh.typ;
    let gesvh_object = dbcon::gesetzesvorhaben::Insert{
        api_id: gesvh.api_id, 
        initiative: gesvh.initiative,
        titel: gesvh.titel,
        trojaner: gesvh.trojaner, 
        verfassungsaendernd: gesvh.verfassungsaendernd, 
        typ: dbschema::gesetzestyp::table
                .filter(dbschema::gesetzestyp::value.eq(
                    format!("{:?}", typ_variant))
                )
                .select(dbschema::gesetzestyp::id)
                .first::<i32>(conn)?
    };
    tracing::debug!("Inserting dbcon::Gesetzesvorhaben");
    let gesvh_id = diesel::insert_into(dbschema::gesetzesvorhaben::table)
            .values(&gesvh_object)
            .returning(dbschema::gesetzesvorhaben::id)
            .get_result::<i32>(conn)?;

    // insert stations
    if gesvh.stationen.is_empty(){
        tracing::warn!("Warning: No station supplied for newly created Gesvh. This is illegal");
        return Err(DatabaseError::MissingFieldForInsert(
            format!("Warning: No station supplied for newly created Gesvh with api id {}. This is illegal",
            gesvh.api_id)
        ));
    }
    tracing::debug!("Preparing Station Insert");
    let mut stmt_inserts = vec![];
    for station in gesvh.stationen{
        tracing::trace!("Handling Station: {:?}", station);
        let station_id = 
        diesel::insert_into(dbschema::station::table)
        .values(&dbcon::station::Insert{
            gesvh_id,
            url: station.url,
            zeitpunkt: station.datum.naive_utc(),
            zuordnung: station.zuordnung,
            stationstyp: dbschema::stationstyp::table
            .filter(dbschema::stationstyp::value.eq(format!("{:?}", station.stationstyp)))
            .select(dbschema::stationstyp::id)
            .first(conn)?,
            parlament: dbschema::parlament::table
            .filter(dbschema::parlament::value.eq(format!("{:?}", station.parlament)))
            .select(dbschema::parlament::id)
            .first(conn)?
        })
        .returning(dbschema::station::id)
        .get_result::<i32>(conn)?;

        let sw_ids = 
            helper_ins_or_ret_schlagwort(station.schlagworte, conn)?;
        diesel::insert_into(dbschema::rel_station_schlagwort::table)
        .values(
            sw_ids.iter().map(
                |id|{
                    use dbschema::rel_station_schlagwort as m;
                    (m::schlagwort_id.eq(id), m::station_id.eq(station_id))
                }
            ).collect::<Vec<_>>()
        )
        .execute(conn)?;
        for dok in station.dokumente{
            let dok_id = helper_create_dokument(dok, conn)?;
            diesel::insert_into(dbschema::rel_station_dokument::table)
            .values(
                (
                    dbschema::rel_station_dokument::dokument_id.eq(dok_id),
                    dbschema::rel_station_dokument::station_id.eq(station_id)
                )
            )
            .execute(conn)?;
        }
        tracing::trace!("Inserting Associated Stellungnahmen");
        for stmt in station.stellungnahmen{
            let stl_dok_id = diesel::insert_into(
                dbschema::dokument::table
            )
            .values(&dbcon::dokument::Insert{
                zusammenfassung: stmt.dokument.zusammenfassung,
                hash: stmt.dokument.hash,
                url: stmt.dokument.url,
                titel: stmt.dokument.titel,
                dokumenttyp_id: dbschema::dokumenttyp::table
                .filter(dbschema::dokumenttyp::value.eq(format!("{:?}", stmt.dokument.typ)))
                .select(dbschema::dokumenttyp::id)
                .first(conn)?
            })
            .returning(dbschema::dokument::id)
            .get_result::<i32>(conn)?;

            let stl_insert = dbcon::stellungnahme::Insert{
                dokument_id: stl_dok_id,
                meinung: stmt.meinung,
                station_id,
                titel: stmt.titel,
                url: stmt.url,
                zeitpunkt: stmt.datum.naive_utc(),
            };
            stmt_inserts.push(stl_insert);
        }
    }
    tracing::trace!("Bulk inserting Statements");
    diesel::insert_into(dbschema::stellungnahme::table)
    .values(stmt_inserts)
    .execute(conn)?;
    // Insert links & notes
    tracing::trace!("Bulk inserting Links");
    diesel::insert_into(dbschema::rel_gesvh_links::table)
    .values(
        gesvh.links.iter()
        .map(|link| {(dbschema::rel_gesvh_links::gesetzesvorhaben_id.eq(gesvh_id), 
        dbschema::rel_gesvh_links::link.eq(link))}).collect::<Vec<_>>()
    )
    .execute(conn)?;
    tracing::trace!("Bulk inserting Notes");
    diesel::insert_into(dbschema::rel_gesvh_notes::table)
    .values(
        gesvh.notes.iter()
        .map(|link| {(dbschema::rel_gesvh_notes::gesetzesvorhaben_id.eq(gesvh_id), 
        dbschema::rel_gesvh_notes::note.eq(link))}).collect::<Vec<_>>()
    )
    .execute(conn)?;
    tracing::info!("Inserted New Gesetzesvorhaben with id {} and api_id {}", gesvh_id, gesvh.api_id);
    Ok(gesvh_id)
}

/// Used to create gesetzesvorhaben & associated data with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    object: api::Gesetzesvorhaben,
) -> std::result::Result<StatusCode, LTZFError> {
    let mut conn = app.pool.get().await.map_err(DatabaseError::from)?;

    let merge_candidates = merge_candidates(&object, &mut conn).await?;
    tracing::info!("Mergeable: {}", merge_candidates.len() == 1);
    tracing::trace!("Merge Candidates: {:?}", merge_candidates);
    if merge_candidates.len() > 1 {
        tracing::warn!("Error: Newly Posted Gesetzesvorhaben has more than one 
        candidate for a merge. It will be inserted as a new entry, please review manually.\n
        Candidate IDs: {:?}\nGesetzesvorhaben: {:?}", &merge_candidates, &object);
        return Err(DatabaseError::MultipleMergeCandidates(merge_candidates, object).into());
    }
    conn.interact(move |conn| {
        conn.transaction(|conn| {
            if merge_candidates.len() == 1 {
                tracing::info!("Merging new Gesetzesvorhaben with {}", merge_candidates[0]);
                let _ = merge_gesvh(object, conn)?;
            } else {
                tracing::info!("Creating a new Gesetzesvorhaben");
                let titel = object.titel.clone();
                let gvh_id = create_gesvh(object, conn)?;
                if merge_candidates.len() > 1{
                    crate::external::send_email(
                        format!("Ambiguous GSVH State: {}", titel), 
                        format!("A new object was posted, but could not be merged due to multiple candidates being found.\n
                        Thus it was inserted as a new object with id {}. The candidates for merging have ids {:?}",
                            gvh_id, merge_candidates), 
                        app.clone());
                }
            }
            Result::<_, DatabaseError>::Ok(())
        })
    })
    .await
    .map_err(DatabaseError::from)??;
    return Ok(StatusCode::CREATED);
}

#[cfg(test)]
mod test{
    use deadpool_diesel::Pool;
    use deadpool_diesel::postgres::Connection;
    use super::*;
    use crate::infra::api;
    macro_rules! pool_creation {
        () => {
            {use deadpool_diesel::Manager;
            let db_url = "postgres://postgres:postgres@localhost/postgres";
            let manager = 
                Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
            Pool::builder(manager).build().unwrap()}
        }
    }

    #[tokio::test]
    async fn create_gesetzesvorhaben()-> Result<(), DatabaseError>{
        let pool = pool_creation!();
        let conn :Connection = pool.get().await.unwrap();
        let input_object = api::Gesetzesvorhaben{
            api_id: uuid::Uuid::now_v7(),
            titel: "Testvorhaben".to_string(),
            verfassungsaendernd: true,
            trojaner: false,
            ids: vec![],
            initiative: "Dingensbums der Dritte".to_string(),
            typ: api::Gesetzestyp::Sonstig,
            links: vec![],
            notes: vec![],
            stationen: vec![
                api::Station{
                    parlament: api::Parlament::BY,
                    schlagworte: vec!["test".to_string(), "abc123".to_string()],
                    url: Some("https://example.com".to_string()),
                    zuordnung: "Ausschuss für Fragen der Testerstellung".to_string(),
                    datum: chrono::Utc::now(),
                    dokumente: vec![],
                    stationstyp: api::Stationstyp::Abgelehnt,
                    stellungnahmen: vec![]
                }
            ]
        };

        let id = 
        conn.interact(|conn|
            create_gesvh(input_object, conn)
        )
        .await??;
        println!("Insert successfull, returning id {}", id);
        Ok(())
    }
}
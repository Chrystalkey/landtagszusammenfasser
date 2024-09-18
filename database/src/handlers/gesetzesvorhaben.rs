use std::collections::HashMap;
use std::sync::Arc;

extern crate diesel_interaction;
use crate::external::no_match_found;
use crate::infra::db::connection as dbcon;
use crate::infra::api::{self, FatOption};
use crate::AppState;
use crate::error::*;
use axum::http::StatusCode;
use diesel::Connection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, OptionalExtension};
use uuid::Uuid;

use crate::async_db;

fn create_gesvh(
    gesvh: api::Gesetzesvorhaben,
    app: Arc<AppState>,
    conn: &mut diesel::pg::PgConnection,
) -> ::std::result::Result<(), DatabaseError> {
    use crate::schema::gesetzesvorhaben as gm;

    let gen_id = Uuid::now_v7();
    
    let federf_db_id = if let Some(value) = gesvh.federfuehrung {
        let value = value.unwrap_data()
        .map_err(|err| DatabaseError::MissingFieldForInsert(format!("{} with value {:?}", err.to_string(), value)))?;
        let name = value.name.clone();
        let res: Vec<i32> = crate::schema::ausschuss::table
            .select(crate::schema::ausschuss::dsl::id)
            .filter(crate::schema::ausschuss::dsl::name.eq(name))
            .load::<i32>(conn)?;
        if res.is_empty(){
            // insert new ausschuss and send email for review
            tracing::warn!("Ausschuss {} not found in database, inserting and sending email for review", value.name.as_str());
            use crate::schema::parlament as pm;
            use crate::schema::ausschuss as am;
            let parl_id : i32 = pm::table
                .select(pm::dsl::id)
                .filter(pm::dsl::kurzname.eq(value.parlament.into_iter().collect::<String>()))
                .first(conn)?;
            let id : i32 = diesel::insert_into(am::table)
            .values(&dbcon::ausschuss::Insert{
                name: value.name.clone(),
                parlament: parl_id,
            })
            .returning(am::dsl::id)
            .get_result(conn)?;
            no_match_found(format!("Ausschuss {} (P: {}) was not found in database, inserted, please review. Id = {}", 
            value.name.as_str(), value.parlament.iter().collect::<String>(), id), 
            app.clone());
            Some(id)
        }else{
            Some(res[0])
        }
    } else {
        None
    };
    let gesvh_typ_id :i32= {
        use crate::schema::gesetzestyp as tm;
        tm::table.select(tm::dsl::id)
        .filter(tm::dsl::value.eq(gesvh.typ.clone()))
        .first(conn)?
    };
    let gesvh_db_insert: dbcon::gesetzesvorhaben::Insert = dbcon::gesetzesvorhaben::Insert {
        api_id: gen_id,
        titel: gesvh.titel,
        verfassungsaendernd: gesvh.verfassungsaendernd,
        trojaner: gesvh.trojaner,
        federf: federf_db_id,
        initiator: gesvh.initiator.clone(),
        typ: gesvh_typ_id,
    };
    let gesvh_db_id :i32 = diesel::insert_into(gm::table)
    .values(gesvh_db_insert)
    .returning(gm::dsl::id)
    .get_result(conn)?;

    // insert links & notes
    diesel::insert_into(crate::schema::further_links::table)
        .values(&gesvh.links.iter().map(|x| dbcon::furtherlinks::Insert{
            gesetzesvorhaben: gesvh_db_id,
            link: x.clone(),
        }).collect::<Vec<dbcon::furtherlinks::Insert>>()
    ).execute(conn)?;
    diesel::insert_into(dbcon::furthernotes::table)
        .values(&gesvh.notes.iter().map(|x| dbcon::furthernotes::Insert{
            gesetzesvorhaben: gesvh_db_id,
            notes: x.clone(),
        }).collect::<Vec<dbcon::furthernotes::Insert>>()
    ).execute(conn)?;
    // insert stationen 
    create_stationen(gesvh_db_id, gesvh.stationen, conn, app)?;
    Ok(())
}

fn create_stationen(gesvh_id: i32, stationen: Vec<FatOption<api::Station, i32>>, conn: &mut diesel::pg::PgConnection, app: Arc<AppState>) -> std::result::Result<(), DatabaseError>{
    // for each station
    for station in stationen{
        let station = station.unwrap_data()
        .map_err(|err| DatabaseError::MissingFieldForInsert(format!("{} with value {:?}", err.to_string(), station)))?;
        let (status, requires_ausschuss) = {
            let id = dbcon::status::table.select(dbcon::status::module::id)
            .filter(dbcon::status::module::value.eq(station.status.clone()))
            .first(conn)
            .optional()?;

            if id.is_none(){
                // insert new status
                let id: i32 = diesel::insert_into(dbcon::status::table)
                .values(&dbcon::status::Insert{
                    value: station.status.clone(),
                })
                .returning(dbcon::status::module::id)
                .get_result(conn)?;
                no_match_found(format!("Status {} was not found in database, inserted, please review. Id = {}", 
                station.status, id), app.clone());
                (id, (station.status == "Parlament: Stellungnahme"))
            }else{(id.unwrap(), (station.status == "Parlament: Stellungnahme"))}
        };

        // insert station, returning id
        let ausschuss_id = match &station.ausschuss{
            None => None,
            Some(data) => {
                let data = data.unwrap_data()
                .map_err(|err| DatabaseError::MissingFieldForInsert(format!("{} with value {:?}", err, data)))?;
                let id = dbcon::ausschuss::table.select(dbcon::ausschuss::module::id)
                .filter(dbcon::ausschuss::module::name.eq(data.name.clone()))
                .first(conn)
                .optional()?;
                if id.is_none(){
                    // insert new ausschuss
                    let id: i32 = diesel::insert_into(dbcon::ausschuss::table)
                    .values(&dbcon::ausschuss::Insert{
                        name: data.name.clone(),
                        parlament: dbcon::parlament::table.select(dbcon::parlament::module::id)
                            .filter(dbcon::parlament::module::kurzname.eq(data.parlament.iter().collect::<String>()))
                            .first(conn)?
                    })
                    .returning(dbcon::ausschuss::module::id)
                    .get_result(conn)?;
                    no_match_found(format!("Ausschuss {} (P: {}) was not found in database, inserted, please review. Id = {}", 
                    data.name, data.parlament.iter().collect::<String>(), id), app.clone());
                    Some(id)
                }else{Some(id.unwrap())}
            }
        };
        let station_id: i32 = diesel::insert_into(dbcon::station::table)
        .values(&dbcon::station::Insert{
            api_id: Uuid::now_v7(),
            parlament: dbcon::parlament::table.select(dbcon::parlament::module::id)
                .filter(dbcon::parlament::module::kurzname.eq(station.parlament.iter().collect::<String>()))
                .first(conn)?,
            gesetzesvorhaben: gesvh_id,
            ausschuss: ausschuss_id, 
            meinungstendenz: station.meinungstenzdenz,
            status,
            datum: station.datum.naive_utc(),
        })
        .returning(dbcon::station::module::id)
        .get_result(conn)?;

        // insert documents, returning id
        let mut autor_inserts = vec![];
        for doc in &station.dokumente{
            let doc = doc.unwrap_data()
            .map_err(|err| DatabaseError::MissingFieldForInsert(format!("{} with value {:?}", err, doc)))?;
            let doktyp_id: i32 = dbcon::dokumententyp::table.select(dbcon::dokumententyp::module::id)
            .filter(dbcon::dokumententyp::module::value.eq(doc.typ.clone()))
            .first(conn)?;
            let api_id = Uuid::now_v7();
            let dok_insert = dbcon::dokument::Insert{
                api_id,
                titel: doc.titel.clone(),
                zsmfassung: doc.zsmfassung.clone(),
                gesetzesvorhaben: gesvh_id,
                hash: doc.hash.clone(),
                identifikator: doc.identifikator.clone(),
                station: station_id,
                last_access: doc.letzter_zugriff.naive_utc(),
                doktyp: doktyp_id,
                url: doc.url.clone(),
            };
            let dok_id : i32 = diesel::insert_into(dbcon::dokument::table)
            .values(&dok_insert)
            .returning(dbcon::dokument::module::id)
            .get_result(conn)?;

            for autor in &doc.autoren{
                let autor_id: i32 = dbcon::autor::table
                .select(dbcon::autor::module::id)
                .filter(dbcon::autor::module::name.eq(autor.0.clone()))
                .filter(dbcon::autor::module::organisation.eq(autor.1.clone()))
                .first::<i32>(conn)
                .optional()?
                .unwrap_or({
                    let autor_id: i32 = diesel::insert_into(dbcon::autor::table)
                    .values(&dbcon::autor::Insert{
                        name: autor.0.clone(),
                        organisation: autor.1.clone(),
                    })
                    .returning(dbcon::autor::module::id)
                    .get_result(conn)?;
                    no_match_found(format!("Autor {} / {} was not found in database, inserted, please review. Id = {}", 
                    autor.0, autor.1, autor_id), app.clone());
                    autor_id
                    });
                autor_inserts.push(dbcon::RelDokAutor{
                    dokument: dok_id,
                    autor: autor_id,
                });
            }
        }

        // insert autoren into rel_dok_autor
        diesel::insert_into(crate::schema::rel_dok_autor::table)
        .values(&autor_inserts)
        .execute(conn)?;
        // insert schlagworte into rel_station_schlagwort
        for schlagwort in &station.schlagworte{
            let schlagwort = schlagwort.to_lowercase();// alle schlagworte sind lowercase, ob man will oder nicht
            let schlagwort_id: i32 = dbcon::schlagwort::table
            .select(dbcon::schlagwort::module::id)
            .filter(dbcon::schlagwort::module::value.eq(schlagwort.clone()))
            .first(conn)
            .optional()?
            .unwrap_or(
                {
                    let schlagwort_id: i32 = diesel::insert_into(dbcon::schlagwort::table)
                    .values(&dbcon::schlagwort::Insert{
                        value: schlagwort.clone(),
                    })
                    .returning(dbcon::schlagwort::module::id)
                    .get_result(conn)?;
                    no_match_found(format!("Schlagwort {} was not found in database, inserted, please review. Id = {}", 
                    schlagwort, schlagwort_id), app.clone());
                    schlagwort_id
                }
            );
            diesel::insert_into(crate::schema::rel_station_schlagwort::table)
            .values(&dbcon::RelStationSchlagwort{
                station: station_id,
                schlagwort: schlagwort_id,
            })
            .execute(conn)?;
        }
    }
    Ok(())
}

/// Used to update gesetzesvorhaben with HTTP PUT
/// This endpoint is supposed to be used by humans who know the data to not have to use the database directly.
pub(crate) async fn put_gesvh(
    _app: Arc<AppState>,
    cupdate: api::CUPUpdate,
    gesvh_id: Uuid,
) -> std::result::Result<api::CUPResponse, LTZFError> {
    todo!("Endpoint to be used for updates by humans, not implemented yet")
}

/// Used to create gesetzesvorhaben & associated data with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    cupdate: api::CUPUpdate,
) -> std::result::Result<StatusCode, LTZFError> {
    let gesvh = cupdate.payload;
    let conn = app.pool.get()
    .await.map_err(DatabaseError::from)?;

    conn.interact( move |conn| {
            conn.transaction::<_, DatabaseError, _>(move |conn|{
                {
                    create_gesvh(gesvh, app, conn)
                }
            }).map_err(LTZFError::from)
        }
    ).await
    // now check for mergeable entries in Gesetzesvorhaben
    // identifieable over title, typ, initiator
    .map_err(DatabaseError::from)??;
    return Ok(StatusCode::CREATED);
}

pub(crate) async fn get_gesvh(app: Arc<AppState>, gesvh_id: Uuid) -> Result<api::WSResponse> {
    let conn = app.pool.get().await.map_err(DatabaseError::from)?;
    let result : dbcon::Gesetzesvorhaben = async_db!(
        conn, first,
        {
            dbcon::gesetzesvorhaben::table
                .filter(dbcon::gesetzesvorhaben::module::api_id.eq(gesvh_id))
        }
    );
    return Ok(
        api::WSResponse{
            id: Uuid::now_v7(),
            payload: api::WSPayload::Gesetzesvorhaben(
                vec![api::Gesetzesvorhaben::construct_from(result, conn).await?]
            ),
        }
    );
}

pub(crate) async fn get_gesvh_filtered(
    app: Arc<AppState>,
    params: HashMap<String, String>,
) -> Result<api::WSResponse> {
    let conn = app.pool.get()
    .await.map_err(DatabaseError::from)?;

    todo!()
}

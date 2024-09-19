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

fn create_gesvh(
    gesvh: api::Gesetzesvorhaben,
    app: Arc<AppState>,
    conn: &mut diesel::pg::PgConnection,
) -> ::std::result::Result<(), DatabaseError> {
    use crate::schema::gesetzesvorhaben as gm;

    let gen_id = Uuid::now_v7();
    tracing::trace!("Generating Id: {}", gen_id);
    
    let federf_db_id = if let Some(value) = gesvh.federfuehrung {
        tracing::trace!("Federführung was Supplied, checking for existence");
        let value = value.unwrap_data()
        .map_err(|err| DatabaseError::MissingFieldForInsert(format!("{} with value {:?}", err.to_string(), value)))?;
        let name = value.name.clone();
        let res: Option<i32> = crate::schema::ausschuss::table
            .select(crate::schema::ausschuss::dsl::id)
            .filter(crate::schema::ausschuss::dsl::name.eq(name))
            .first::<i32>(conn)
            .optional()?;
        if res.is_none(){
            // insert new ausschuss and send email for review
            tracing::warn!("Ausschuss {} not found in database, inserting and sending email for review", value.name.as_str());
            use crate::schema::parlament as pm;
            use crate::schema::ausschuss as am;
            tracing::trace!("Checking wether Parlament {} for exists", value.parlament.iter().collect::<String>());
            let parl_id : i32 = pm::table
                .select(pm::dsl::id)
                .filter(pm::dsl::kurzname.eq(value.parlament.into_iter().collect::<String>()))
                .first(conn)?;
            tracing::trace!("Parlament found");
            tracing::trace!("Inserting new Ausschuss");
            let id : i32 = diesel::insert_into(am::table)
            .values(&dbcon::ausschuss::Insert{
                name: value.name.clone(),
                parlament: parl_id,
            })
            .returning(am::dsl::id)
            .get_result(conn)?;
            tracing::trace!("Newly inserted Ausschuss has id: {}", id);
            tracing::debug!("Ausschuss {} (P: {}) was not found in database, inserted, please review. Id = {}", 
            value.name.as_str(), value.parlament.iter().collect::<String>(), id);
            no_match_found(format!("Ausschuss {} (P: {}) was not found in database, inserted, please review. Id = {}", 
            value.name.as_str(), value.parlament.iter().collect::<String>(), id), 
            app.clone());
            Some(id)
        }else{
            tracing::debug!("Federführung found in database: id = {}", res.unwrap());
            res
        }
    } else {
        None
    };
    let gesvh_typ_id :i32= {
        tracing::debug!("Trying to find Gesetzestyp in Database");
        use crate::schema::gesetzestyp as tm;
        tm::table.select(tm::dsl::id)
        .filter(tm::dsl::value.eq(gesvh.typ.clone()))
        .first(conn)?
    };
    tracing::debug!("Gesetzestyp found, id = {}", gesvh_typ_id);
    let gesvh_db_insert: dbcon::gesetzesvorhaben::Insert = dbcon::gesetzesvorhaben::Insert {
        api_id: gen_id,
        titel: gesvh.titel,
        verfassungsaendernd: gesvh.verfassungsaendernd,
        trojaner: gesvh.trojaner,
        federf: federf_db_id,
        initiator: gesvh.initiator.clone(),
        typ: gesvh_typ_id,
    };
    tracing::debug!("Inserting Gesetzesvorhaben into Database as struct: {:?}", gesvh_db_insert);
    let gesvh_db_id :i32 = diesel::insert_into(gm::table)
    .values(gesvh_db_insert)
    .returning(gm::dsl::id)
    .get_result(conn)?;
    tracing::debug!("Insertion successful, id = {}", gesvh_db_id);

    // insert links & notes
    tracing::trace!("Inserting Further Links");
    diesel::insert_into(crate::schema::further_links::table)
        .values(&gesvh.links.iter().map(|x| dbcon::furtherlinks::Insert{
            gesetzesvorhaben: gesvh_db_id,
            link: x.clone(),
        }).collect::<Vec<dbcon::furtherlinks::Insert>>()
    ).execute(conn)?;
    tracing::trace!("Inserting Further Notes");
    diesel::insert_into(dbcon::furthernotes::table)
        .values(&gesvh.notes.iter().map(|x| dbcon::furthernotes::Insert{
            gesetzesvorhaben: gesvh_db_id,
            notes: x.clone(),
        }).collect::<Vec<dbcon::furthernotes::Insert>>()
    ).execute(conn)?;
    // insert stationen 
    tracing::trace!("Inserting Stationen");
    create_stationen(gesvh_db_id, gesvh.stationen, conn, app)?;
    Ok(())
}

fn create_stationen(gesvh_id: i32, stationen: Vec<FatOption<api::Station, i32>>, conn: &mut diesel::pg::PgConnection, app: Arc<AppState>) -> std::result::Result<(), DatabaseError>{
    if stationen.is_empty(){
        tracing::warn!("No Stationen supplied, is a mandatory field");
        return Err(
            DatabaseError::MissingFieldForInsert("No Stationen supplied, is a mandatory field".to_string())
        );
    }
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
                (id, (station.status == "Parlament: Beschlussempfehlung"))
            }else{(id.unwrap(), (station.status == "Parlament: Beschlussempfehlung"))}
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
                .filter(dbcon::parlament::module::kurzname.eq(station.parlament.clone()))
                .first(conn)?,
            gesetzesvorhaben: gesvh_id,
            ausschuss: 
            if requires_ausschuss && ausschuss_id.is_none() {
                return Err(DatabaseError::MissingFieldForInsert(format!("Inserting a Station of this type requires an Ausschuss to be supplied.")));
            }else{ausschuss_id}, 
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

/// Used to create gesetzesvorhaben & associated data with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    cupdate: api::CUPUpdate,
) -> std::result::Result<StatusCode, LTZFError> {
    let gesvh = cupdate.payload;
    let conn = app.pool.get()
    .await.map_err(DatabaseError::from)?;
    tracing::debug!("Starting Insertion Transaction");
    conn.interact( move |conn| {
            conn.transaction::<_, DatabaseError, _>(move |conn|{
                {
                    create_gesvh(gesvh, app, conn)
                }
            }).map_err(|err| 
                {tracing::warn!("Transaction cancelled, rolled back");LTZFError::from(err)})
        }
    ).await
    // now check for mergeable entries in Gesetzesvorhaben
    // identifieable over title, typ, initiator
    .map_err(DatabaseError::from)??;
    tracing::debug!("Finished Insertion Transaction");
    tracing::info!("Inserted New Gesetzesvorhaben into Database");
    return Ok(StatusCode::CREATED);
}

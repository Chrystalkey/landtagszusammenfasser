use std::sync::Arc;

extern crate diesel_interaction;
extern crate diesel_interaction;
use crate::error::*;
use crate::infra::api as ifapi;
use crate::infra::db::connection as dbcon;
use crate::AppState;
use diesel::RunQueryDsl;
use diesel::RunQueryDsl;
use ifapi::{CUPPayload, CUPResponse, CUPResponsePayload, DatabaseInteraction};
use uuid::Uuid;
macro_rules! handle_retrieval_error {
    ($field:expr, $name:expr, $conn:ident, $app:ident) => {
        match &$field {
            Some(thing) => match thing.fetch_id(&mut $conn).await {
                Ok(id) => Some(id),
                Err(LTZFError::RetrievalError(RetrievalError::NoMatch)) => {
                    crate::external::no_match_found(
                        format!(
                            "No match was found for field `{}` using this query: {:?}",
                            $name, &$field
                        ),
                        $app.clone(),
                    )
                    .await;
                    None
                }
                Err(error) => return Err(error),
            },
            None => None,
        }
    };
}
async fn update_gesvh(db_id: i32, gesvh: ifapi::Gesetzesvorhaben, app: Arc<AppState>, mut conn: deadpool_diesel::postgres::Connection) -> Result<ifapi::Gesetzesvorhaben>{

    let feder = handle_retrieval_error!(gesvh.federfuehrung, "Federführung", conn, app);
    let initiat = handle_retrieval_error!(gesvh.initiator, "Initiator", conn, app);
    let gesvh_copy = gesvh.clone();

    let update_structure = dbcon::gesetzesvorhaben::Update{
        ext_id: gesvh.ext_id,
        titel: gesvh.titel,
        verfassungsaendernd: gesvh.verfassungsaendernd,
        id_gesblatt: Some(gesvh.id_gesblatt),
        off_titel: gesvh.off_titel,
        url_gesblatt: Some(gesvh.url_gesblatt),
        trojaner: Some(gesvh.trojaner),
        feder: Some(feder),
        initiat: Some(initiat),
    };
    let result = dbcon::gesetzesvorhaben::update(&mut conn, db_id, &update_structure).await.map_err(DatabaseError::from)?;
    if result != 1{
        return Err(crate::error::DatabaseError::DatabaseError(
            format!("Update Failed, affecting {} Rows", result)).into()
        );
    }
    return Ok(gesvh_copy);
}

async fn insert_dokuments(
    gesvh_id: i32,
    dokumente: Vec<ifapi::Dokument>,
    conn: deadpool_diesel::postgres::Connection,
) -> Result<Vec<Uuid>> {
    // find all doktyp ids
    let mut dok_inserts = vec![];
    for entry in dokumente.iter() {
        let dtid = dbcon::dokumenttypen::select_matching(&mut conn,
            dbcon::dokumenttypen::Update{
                name: Some(entry.typ.clone())
            }
        ).await.map_err(DatabaseError::from)?.first().unwrap().id;
        let insert = 
        dbcon::dokumente::Insert{
            gesetzesvorhaben: gesvh_id,
            doktyp: Some(dtid),
            url: entry.url.clone(),
            ext_id: entry.ext_id,
            filetype: entry.filetype.clone(),
            hash: entry.hash.clone(),
            off_id: entry.off_id.clone(),
            path: entry.path.clone(),
            beschreibung: entry.beschreibung.clone()
            accessed_at: entry.letzter_zugriff,
            created_at: entry.erstellt_am,
        };
    }
    // insert dokumente
    todo!()
}

async fn create_gesvh(
    gesvh: ifapi::Gesetzesvorhaben, 
    app: Arc<AppState>, 
    mut conn: deadpool_diesel::postgres::Connection) -> Result<Uuid> {
    let gen_id = Uuid::now_v7();
async fn update_gesvh(db_id: i32, gesvh: ifapi::Gesetzesvorhaben, app: Arc<AppState>, mut conn: deadpool_diesel::postgres::Connection) -> Result<ifapi::Gesetzesvorhaben>{

    let feder = handle_retrieval_error!(gesvh.federfuehrung, "Federführung", conn, app);
    let initiat = handle_retrieval_error!(gesvh.initiator, "Initiator", conn, app);
    let gesvh_copy = gesvh.clone();

    let update_structure = dbcon::gesetzesvorhaben::Update{
        ext_id: gesvh.ext_id,
        titel: gesvh.titel,
        verfassungsaendernd: gesvh.verfassungsaendernd,
        id_gesblatt: Some(gesvh.id_gesblatt),
        off_titel: gesvh.off_titel,
        url_gesblatt: Some(gesvh.url_gesblatt),
        trojaner: Some(gesvh.trojaner),
        feder: Some(feder),
        initiat: Some(initiat),
    };
    let result = dbcon::gesetzesvorhaben::update(&mut conn, db_id, &update_structure).await.map_err(DatabaseError::from)?;
    if result != 1{
        return Err(crate::error::DatabaseError::DatabaseError(
            format!("Update Failed, affecting {} Rows", result)).into()
        );
    }
    return Ok(gesvh_copy);
}

async fn insert_dokuments(
    gesvh_id: i32,
    dokumente: Vec<ifapi::Dokument>,
    conn: deadpool_diesel::postgres::Connection,
) -> Result<Vec<Uuid>> {
    // find all doktyp ids
    let mut dok_inserts = vec![];

    for entry in dokumente.iter() {
        let dtid = diesel::
    }
    // insert dokumente
    todo!()
}

async fn create_gesvh(
    gesvh: ifapi::Gesetzesvorhaben, 
    app: Arc<AppState>, 
    mut conn: deadpool_diesel::postgres::Connection) -> Result<Uuid> {
    let gen_id = Uuid::now_v7();
        let feder = handle_retrieval_error!(gesvh.federfuehrung, "Federführung", conn, app);
        let initiat = handle_retrieval_error!(gesvh.initiator, "Initiator", conn, app);

        let ins_gesvh = dbcon::gesetzesvorhaben::Insert {
            ext_id: gen_id,
            off_titel: gesvh.off_titel.map_or(
                Err(DatabaseError::DatabaseError(
                    "off_titel is a required field".to_owned(),
                )),
                |x| Ok(x),
            )?,
            titel: gesvh.titel.map_or(
                Err(DatabaseError::DatabaseError(
                    "Titel is a required field".to_owned(),
                )),
                |x| Ok(x),
            )?,
            verfassungsaendernd: gesvh.verfassungsaendernd.map_or(
                Err(DatabaseError::DatabaseError(
                    "Verfassungsändernd is a required field".to_owned(),
                )),
                |x| Ok(x),
            )?,
            id_gesblatt: gesvh.id_gesblatt,
            url_gesblatt: gesvh.url_gesblatt,
            trojaner: gesvh.trojaner,
            feder,
            feder,
            initiat,
        };
        let gesvh_id = dbcon::gesetzesvorhaben::insert(&mut conn, ins_gesvh)
        .await
        .map_err(DatabaseError::from)?;
        if gesvh.schlagworte.len() > 0 {
            let mut sw_ids = vec![];
            for sw in gesvh.schlagworte{
                let req_template = dbcon::schlagworte::Update{
                    schlagwort: Some(sw.clone()),
                    beschreibung : None
                };
                let sw_result = 
                dbcon::schlagworte::select_matching(&mut conn, req_template.clone())
                .await
                .map_err(DatabaseError::from)?;
                let sw_id = if sw_result.len() == 0{
                    crate::external::no_match_found(
                        format!(
                            "No Schlagwort `{}` found. Creating it without description, please review.",
                            sw.as_str()
                        )
                        , app.clone()).await;
                        dbcon::schlagworte::insert(
                            &mut conn,
                            dbcon::schlagworte::Insert{
                                schlagwort: sw,
                                beschreibung: "".to_owned()
                            }
                        ).await.map_err(DatabaseError::from)?
                }else{
                    sw_result.first().unwrap().id
                };
                sw_ids.push(sw_id);
            }
            use crate::schema::rel_ges_schlagworte::dsl as module;
            use crate::schema::rel_ges_schlagworte::table;
            use diesel::ExpressionMethods;
            let records: Vec<_> = sw_ids
            .iter()
            .map(|&x| (module::gesetzesvorhaben.eq(gesvh_id), module::schlagwort.eq(x)))
            .collect();
            conn.interact(move |conn|{
                diesel::insert_into(table)
                .values(&records)
                .execute(conn)
            }).await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
            .map_err(DatabaseError::from)?;
        }
        if gesvh.status.len() > 0{
            // insert status
            let mut stat_ids = vec![];
            for entry in gesvh.status{
                let pid = 
                ifapi::fetch_parlament_id(entry.parlament.clone(), 
                &mut conn).await?;
                let req_template = dbcon::status::Update{
                    name: Some(entry.name.clone()),
                    parlament: Some(Some(pid))
                };
                let stat_results = 
                dbcon::status::select_matching(&mut conn, req_template.clone())
                .await
                .map_err(DatabaseError::from)?;

                let status_id = if stat_results.len() == 0{
                    crate::external::no_match_found(
                        format!(
                            "No Status `{}` in Parlament `{}` found for Gesetzesvorhaben `{}`. Creating it automatically, please review entry.",
                            entry.name.as_str(),
                            entry.parlament.iter().collect::<String>(),
                            gen_id
                        ), app.clone()
                    )
                    .await;

                    dbcon::status::insert(
                        &mut conn,
                        dbcon::status::Insert{
                            name: entry.name,
                            parlament: Some(pid)
                        }
                    ).await
                    .map_err(DatabaseError::from)?
                }else{
                    stat_results.first().unwrap().id
                };
                stat_ids.push((status_id, entry.datum));
            }
            use crate::schema::rel_ges_status::dsl as module;
            use crate::schema::rel_ges_status::table;
            use diesel::ExpressionMethods;
            let records: Vec<_> = stat_ids
            .iter()
            .map(|&(id, date)| (module::gesetzesvorhaben.eq(gesvh_id), 
            module::status.eq(id), module::datum.eq(date)
        ))
            .collect();
            conn.interact(move |conn|{
                diesel::insert_into(table)
                .values(&records)
                .execute(conn)
            }).await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
            .map_err(DatabaseError::from)?;
        }
        if gesvh.eigenschaften.len() > 0{
            // insert eigenschaften
            let mut eig_ids = vec![];
            for entry in gesvh.eigenschaften{
                let req_template = dbcon::gesetzeseigenschaften::Update{
                    eigenschaft: Some(entry.clone())
                };
                let retrieval_results = 
                dbcon::gesetzeseigenschaften::select_matching(&mut conn, req_template.clone())
                .await
                .map_err(DatabaseError::from)?;

                let status_id = if retrieval_results.len() == 0{
                    crate::external::no_match_found(
                        format!(
                            "No Gesetzeseigenschaft called `{}` found for Gesetzesvorhaben with UUID `{}`. 
                            Creating it automatically, please review entry.",
                            entry.as_str(),
                            gen_id
                        ), app.clone()
                    )
                    .await;

                    dbcon::gesetzeseigenschaften::insert(
                        &mut conn,
                        dbcon::gesetzeseigenschaften::Insert{
                            eigenschaft: entry
                        }
                    ).await
                    .map_err(DatabaseError::from)?
                }else{
                    retrieval_results.first().unwrap().id
                };
                eig_ids.push(status_id);
            }
            use crate::schema::rel_ges_eigenschaft::dsl as module;
            use crate::schema::rel_ges_eigenschaft::table;
            use diesel::ExpressionMethods;
            let records: Vec<_> = eig_ids
            .iter()
            .map(|&id| (module::gesetzesvorhaben.eq(gesvh_id),module::eigenschaft.eq(id)))
            .collect();
            conn.interact(move |conn|{
                diesel::insert_into(table)
                .values(&records)
                .execute(conn)
            }).await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
        let gesvh_id = dbcon::gesetzesvorhaben::insert(&mut conn, ins_gesvh)
        .await
        .map_err(DatabaseError::from)?;
        if gesvh.schlagworte.len() > 0 {
            let mut sw_ids = vec![];
            for sw in gesvh.schlagworte{
                let req_template = dbcon::schlagworte::Update{
                    schlagwort: Some(sw.clone()),
                    beschreibung : None
                };
                let sw_result = 
                dbcon::schlagworte::select_matching(&mut conn, req_template.clone())
                .await
                .map_err(DatabaseError::from)?;
                let sw_id = if sw_result.len() == 0{
                    crate::external::no_match_found(
                        format!(
                            "No Schlagwort `{}` found. Creating it without description, please review.",
                            sw.as_str()
                        )
                        , app.clone()).await;
                        dbcon::schlagworte::insert(
                            &mut conn,
                            dbcon::schlagworte::Insert{
                                schlagwort: sw,
                                beschreibung: "".to_owned()
                            }
                        ).await.map_err(DatabaseError::from)?
                }else{
                    sw_result.first().unwrap().id
                };
                sw_ids.push(sw_id);
            }
            use crate::schema::rel_ges_schlagworte::dsl as module;
            use crate::schema::rel_ges_schlagworte::table;
            use diesel::ExpressionMethods;
            let records: Vec<_> = sw_ids
            .iter()
            .map(|&x| (module::gesetzesvorhaben.eq(gesvh_id), module::schlagwort.eq(x)))
            .collect();
            conn.interact(move |conn|{
                diesel::insert_into(table)
                .values(&records)
                .execute(conn)
            }).await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
            .map_err(DatabaseError::from)?;
        }
        if gesvh.status.len() > 0{
            // insert status
            let mut stat_ids = vec![];
            for entry in gesvh.status{
                let pid = 
                ifapi::fetch_parlament_id(entry.parlament.clone(), 
                &mut conn).await?;
                let req_template = dbcon::status::Update{
                    name: Some(entry.name.clone()),
                    parlament: Some(Some(pid))
                };
                let stat_results = 
                dbcon::status::select_matching(&mut conn, req_template.clone())
                .await
                .map_err(DatabaseError::from)?;

                let status_id = if stat_results.len() == 0{
                    crate::external::no_match_found(
                        format!(
                            "No Status `{}` in Parlament `{}` found for Gesetzesvorhaben `{}`. Creating it automatically, please review entry.",
                            entry.name.as_str(),
                            entry.parlament.iter().collect::<String>(),
                            gen_id
                        ), app.clone()
                    )
                    .await;

                    dbcon::status::insert(
                        &mut conn,
                        dbcon::status::Insert{
                            name: entry.name,
                            parlament: Some(pid)
                        }
                    ).await
                    .map_err(DatabaseError::from)?
                }else{
                    stat_results.first().unwrap().id
                };
                stat_ids.push((status_id, entry.datum));
            }
            use crate::schema::rel_ges_status::dsl as module;
            use crate::schema::rel_ges_status::table;
            use diesel::ExpressionMethods;
            let records: Vec<_> = stat_ids
            .iter()
            .map(|&(id, date)| (module::gesetzesvorhaben.eq(gesvh_id), 
            module::status.eq(id), module::datum.eq(date)
        ))
            .collect();
            conn.interact(move |conn|{
                diesel::insert_into(table)
                .values(&records)
                .execute(conn)
            }).await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
            .map_err(DatabaseError::from)?;
        }
        if gesvh.eigenschaften.len() > 0{
            // insert eigenschaften
            let mut eig_ids = vec![];
            for entry in gesvh.eigenschaften{
                let req_template = dbcon::gesetzeseigenschaften::Update{
                    eigenschaft: Some(entry.clone())
                };
                let retrieval_results = 
                dbcon::gesetzeseigenschaften::select_matching(&mut conn, req_template.clone())
                .await
                .map_err(DatabaseError::from)?;

                let status_id = if retrieval_results.len() == 0{
                    crate::external::no_match_found(
                        format!(
                            "No Gesetzeseigenschaft called `{}` found for Gesetzesvorhaben with UUID `{}`. 
                            Creating it automatically, please review entry.",
                            entry.as_str(),
                            gen_id
                        ), app.clone()
                    )
                    .await;

                    dbcon::gesetzeseigenschaften::insert(
                        &mut conn,
                        dbcon::gesetzeseigenschaften::Insert{
                            eigenschaft: entry
                        }
                    ).await
                    .map_err(DatabaseError::from)?
                }else{
                    retrieval_results.first().unwrap().id
                };
                eig_ids.push(status_id);
            }
            use crate::schema::rel_ges_eigenschaft::dsl as module;
            use crate::schema::rel_ges_eigenschaft::table;
            use diesel::ExpressionMethods;
            let records: Vec<_> = eig_ids
            .iter()
            .map(|&id| (module::gesetzesvorhaben.eq(gesvh_id),module::eigenschaft.eq(id)))
            .collect();
            conn.interact(move |conn|{
                diesel::insert_into(table)
                .values(&records)
                .execute(conn)
            }).await
            .map_err(diesel_interaction::DieselInteractionError::from)
            .map_err(DatabaseError::from)?
            .map_err(DatabaseError::from)?;
        }
        if gesvh.dokumente.len() > 0{
            // insert dokumente
            insert_dokuments(gesvh_id, gesvh.dokumente, conn).await?;
        }
        return Ok(gen_id);
}

/// Used to update gesetzesvorhaben with HTTP PUT
pub(crate) async fn put_gesvh(
    app: Arc<AppState>,
    cupdate: ifapi::CUPUpdate,
    gesvh_id: Uuid,
) -> std::result::Result<CUPResponse, LTZFError> {
    let gesvh_update = match cupdate.payload{
        CUPPayload::GesVH(gevh_update) => gevh_update,
        _ => {return Err(LTZFError::WrongEndpoint("Used endpoint `gesetzesvorhaben` with PUT, but supplied a different payload structure".to_owned()))}
    };
    if  gesvh_update.ext_id.is_some() && 
        gesvh_update.ext_id.unwrap() != gesvh_id {
        return Err(
            LTZFError::ParsingError(
                ParsingError::Internal(
                    "Endpoint was called with a different Uuid than the one contained in the payload".to_owned()
                )
            )
        )
    }
    todo!()
}

/// Used to create gesetzesvorhaben with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    cupdate: ifapi::CUPUpdate,
) -> std::result::Result<CUPResponse, LTZFError> {
    let gesvh = if let CUPPayload::GesVH(gesvh) = cupdate.payload {
        gesvh
    } else {
        return Err(LTZFError::WrongEndpoint("Gesetzesvorhaben".to_owned()));
    };
    let conn = app.pool.get().await.map_err(DatabaseError::from)?;
    if gesvh.ext_id.is_some() {
        return Err(
            LTZFError::WrongEndpoint(
                format!("Used POST on Gesetzesvorhaben, but supplied ext_id {}", 
                    gesvh.ext_id.unwrap()
                )
            )
        )
    }
    // no id supplied, so assuming create with the existing ones
    let gen_id = create_gesvh(gesvh, app, conn).await?;
    let response = CUPResponse {
        msg_id: Uuid::now_v7(),
        timestamp: chrono::Utc::now(),
        responding_to: cupdate.msg_id,
        payload: CUPResponsePayload {
            data: CUPPayload::GesVH(ifapi::Gesetzesvorhaben {
                ext_id: Some(gen_id),
                ..Default::default()
            }),
            state: ifapi::CUPRessourceState::Created,
        },
    };
    return Ok(response);
    // if more than one match: error on ambiguous data, let a human decide. handled internally through email
        }
        if gesvh.dokumente.len() > 0{
            // insert dokumente
            insert_dokuments(gesvh_id, gesvh.dokumente, conn).await?;
        }
        return Ok(gen_id);
}

/// Used to update gesetzesvorhaben with HTTP PUT
pub(crate) async fn put_gesvh(
    app: Arc<AppState>,
    cupdate: ifapi::CUPUpdate,
    gesvh_id: Uuid,
) -> std::result::Result<CUPResponse, LTZFError> {
    let gesvh_update = match cupdate.payload{
        CUPPayload::GesVH(gevh_update) => gevh_update,
        _ => {return Err(LTZFError::WrongEndpoint("Used endpoint `gesetzesvorhaben` with PUT, but supplied a different payload structure".to_owned()))}
    };
    if  gesvh_update.ext_id.is_some() && 
        gesvh_update.ext_id.unwrap() != gesvh_id {
        return Err(
            LTZFError::ParsingError(
                ParsingError::Internal(
                    "Endpoint was called with a different Uuid than the one contained in the payload".to_owned()
                )
            )
        )
    }
    todo!()
}

/// Used to create gesetzesvorhaben with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    cupdate: ifapi::CUPUpdate,
) -> std::result::Result<CUPResponse, LTZFError> {
    let gesvh = if let CUPPayload::GesVH(gesvh) = cupdate.payload {
        gesvh
    } else {
        return Err(LTZFError::WrongEndpoint("Gesetzesvorhaben".to_owned()));
    };
    let conn = app.pool.get().await.map_err(DatabaseError::from)?;
    if gesvh.ext_id.is_some() {
        return Err(
            LTZFError::WrongEndpoint(
                format!("Used POST on Gesetzesvorhaben, but supplied ext_id {}", 
                    gesvh.ext_id.unwrap()
                )
            )
        )
    }
    // no id supplied, so assuming create with the existing ones
    let gen_id = create_gesvh(gesvh, app, conn).await?;
    let response = CUPResponse {
        msg_id: Uuid::now_v7(),
        timestamp: chrono::Utc::now(),
        responding_to: cupdate.msg_id,
        payload: CUPResponsePayload {
            data: CUPPayload::GesVH(ifapi::Gesetzesvorhaben {
                ext_id: Some(gen_id),
                ..Default::default()
            }),
            state: ifapi::CUPRessourceState::Created,
        },
    };
    return Ok(response);
    // if more than one match: error on ambiguous data, let a human decide. handled internally through email
}

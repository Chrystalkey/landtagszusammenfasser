use std::sync::Arc;

extern crate diesel_interaction;
use crate::external::no_match_found;
use crate::infra::api::collectors as clapi;
use crate::infra::api::webservice as wsapi;
use crate::infra::db::connection as dbcon;
use crate::AppState;
use crate::{error::*, router::GetGesvhQueryFilters};
use clapi::CUPResponse;
use diesel::Connection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

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

macro_rules! get_id_or_insert {
    ($module:path, $field:expr) => {
        if let Some(value) = $field {
            
        }
        else{None}
    }
}
#[allow(dead_code)]
async fn update_gesvh(
    db_id: i32,
    gesvh: clapi::Gesetzesvorhaben,
    app: Arc<AppState>,
    mut conn: deadpool_diesel::postgres::Connection,
) -> Result<clapi::Gesetzesvorhaben> {
    todo!();
}

fn insert_dokumente(
    gesvh_id: i32,
    dokumente: Vec<clapi::Dokument>,
    conn: &mut diesel::pg::PgConnection,
    app: Arc<AppState>,
) -> std::result::Result<Vec<Uuid>, DatabaseError> {
    // insert dokumente
    use crate::schema::dokumente::table as dok_table;
    use crate::schema::dokumenttypen::dsl as dt_module;
    use crate::schema::dokumenttypen::table as dt_table;

    // find all doktyp ids
    let mut dok_inserts = vec![];

    for entry in dokumente {
        let dtid  : i32 = {
            let dtid  = dt_table
                .select(dt_module::id)
                .filter(dt_module::name.eq(entry.typ.clone()))
                .load::<i32>(conn)?;
            if dtid.is_empty(){
                // insert new doktyp and send email for review
                tracing::warn!("Dokumenttyp {} not found in database, inserting and sending email for review", entry.typ.as_str());
                let id : i32 = diesel::insert_into(dt_table)
                    .values(&dbcon::dokumenttyp::Insert{
                        name: entry.typ.clone(),
                    })
                    .returning(dt_module::id)
                    .get_result(conn)?;
                no_match_found(format!("Dokumenttyp {} was not found in database, inserted, please review, new Id = {}", 
                entry.typ.as_str(), id), 
                app.clone());
                id
            }else{
                *dtid.first().unwrap()
            }
        };
        dok_inserts.push(dbcon::dokument::Insert {
            ext_id: Uuid::now_v7(),
            accessed_at: entry.letzter_zugriff.clone().naive_utc(),
            created_at: required_field!(entry.erstellt_am.map(|x| x.naive_utc())),
            doktyp: Some(dtid),
            filetype: entry.file_type.clone(),
            gesetzesvorhaben: Some(gesvh_id),
            url: required_field!(entry.url),
            hash: required_field!(entry.hash),
            off_id: required_field!(entry.off_id),
            path: entry.pfad.clone(),
        });
    }
    let created_dok_ids: Vec<Uuid> = dok_inserts.iter().map(|x| x.ext_id).collect();
    diesel::insert_into(dok_table)
    .values(&dok_inserts)
    .execute(conn)?;
    // construct response
    Ok(created_dok_ids)
}

fn create_gesvh(
    gesvh: clapi::Gesetzesvorhaben,
    app: Arc<AppState>,
    conn: &mut diesel::pg::PgConnection,
) -> ::std::result::Result<clapi::Gesetzesvorhaben, DatabaseError> {
    use crate::schema::gesetzesvorhaben as gm;

    let gen_id = Uuid::now_v7();
    
    let federf_db_id = if let Some(value) = gesvh.federfuehrung {
        let name = value.name.clone();
        let res: Vec<i32> = crate::schema::ausschuesse::table
            .select(crate::schema::ausschuesse::dsl::id)
            .filter(crate::schema::ausschuesse::dsl::name.eq(name))
            .load::<i32>(conn)?;
        if res.is_empty(){
            // insert new ausschuss and send email for review
            tracing::warn!("Ausschuss {} not found in database, inserting and sending email for review", value.name.as_str());
            use crate::schema::parlamente as pm;
            use crate::schema::ausschuesse as am;
            let parl_id : i32 = pm::table
                .select(pm::dsl::id)
                .filter(pm::dsl::kurzname.eq(value.parlament.into_iter().collect::<String>()))
                .first(conn)?;
            let id : i32 = diesel::insert_into(am::table)
            .values(&dbcon::ausschuss::Insert{
                name: value.name.clone(),
                parlament: Some(parl_id),
            })
            .returning(am::dsl::id)
            .get_result(conn)?;
            no_match_found(format!("Ausschuss {} (P: {}) was not found in database, inserted, please review, new Id = {}", 
            value.name.as_str(), value.parlament.iter().collect::<String>(), id), 
            app.clone());
            Some(id)
        }else{
            Some(res[0])
        }
    } else {
        None
    };
    let init_db_id = if let Some(value) = gesvh.initiator {
        let val_clone = value.clone();
        use crate::schema::initiatoren as im;
        let res = {
            let mut query = im::table.into_boxed().select(im::dsl::id);
            query = query.filter(im::dsl::name.eq(val_clone.name));
            if let Some(org) = &val_clone.organisation {
                query= query.filter(im::dsl::organisation.eq(org));
            }
            if let Some(url) = &val_clone.url {
                query= query.filter(im::dsl::url.eq(url));
            }
            query.load::<i32>(conn)?
        };
        if res.is_empty(){
            // insert new initiator and send email for review
            tracing::warn!("Initiator {} not found in database, inserting and sending email for review", &value.name);
            let org = required_field!(value.organisation.clone());
            let url = required_field!(value.url.clone());
            let name = value.name.clone();
            let id: i32 = diesel::insert_into(im::table)
            .values(&dbcon::initiator::Insert{
                name: name.clone(),
                organisation: org,
                url: url,
            })
            .returning(im::dsl::id)
            .get_result(conn)?;
            no_match_found(format!("Initiator {} (O: {}/U: {}) not found in database, inserted, please review, new Id = {}", 
            name, value.organisation.unwrap(), value.url.unwrap(), id), 
            app.clone());
            Some(id)
        }else{
            Some(res[0])
        }
    } else {
        None
    };
    let gesvh_db_insert: dbcon::gesetzesvorhaben::Insert = dbcon::gesetzesvorhaben::Insert {
        ext_id: gen_id,
        titel: required_field!(gesvh.titel),
        off_titel: required_field!(gesvh.off_titel),
        verfassungsaendernd: required_field!(gesvh.verfassungsaendernd),
        trojaner: required_field!(gesvh.trojaner),
        id_gesblatt: gesvh.id_gesblatt.clone(),
        url_gesblatt: gesvh.url_gesblatt.clone(),
        feder: federf_db_id,
        initiat: init_db_id,
    };
    let gesvh_db_id :i32 = diesel::insert_into(gm::table)
    .values(gesvh_db_insert)
    .returning(gm::dsl::id)
    .get_result(conn)?;

    // insert status
    if let Some(value) = gesvh.status {
        let name = value.name.clone();
        let res: Vec<i32> = crate::schema::status::table
            .select(crate::schema::status::dsl::id)
            .filter(crate::schema::status::dsl::name.eq(name))
            .load::<i32>(conn)?;
        let stat_id = if res.is_empty(){
            // insert new status and send email for review
            tracing::warn!("Status {} not found in database, inserting and sending email for review", value.name.as_str());
            use crate::schema::parlamente as pm;
            use crate::schema::status as sm;
            let parl_id : i32 = pm::table
                .select(pm::dsl::id)
                .filter(pm::dsl::kurzname.eq(value.parlament.into_iter().collect::<String>()))
                .first(conn)?;
            let id : i32 = diesel::insert_into(sm::table)
                .values(&dbcon::status::Insert{
                    name: value.name.clone(),
                    parlament: Some(parl_id),
                })
                .returning(sm::dsl::id)
                .get_result(conn)?;
            no_match_found(format!("Status {} (P: {}) was not found in database, inserted, please review, new Id = {}", 
            value.name.as_str(), value.parlament.iter().collect::<String>(), id), 
            app.clone());
            id
        }else{
            res[0]
        };
        diesel::insert_into(crate::schema::rel_ges_status::table)
            .values(&dbcon::RelGesStatus{
                gesetzesvorhaben: gesvh_db_id,
                status: stat_id,
                datum: value.datum.naive_utc()
            })
            .execute(conn)?;
    }
    // insert schlagworte
    if gesvh.schlagworte.len() > 0 {
        let mut schlagworte = vec![];
        for schlag in gesvh.schlagworte {
            let res: Vec<i32> = crate::schema::schlagworte::table
                .select(crate::schema::schlagworte::dsl::id)
                .filter(crate::schema::schlagworte::dsl::schlagwort.eq(schlag.clone()))
                .load::<i32>(conn)?;
            let schlag_id = if res.is_empty(){
                // insert new schlagwort and send email for review
                tracing::warn!("Schlagwort {} not found in database, inserting and sending email for review", schlag.as_str());
                let id : i32 = diesel::insert_into(crate::schema::schlagworte::table)
                    .values(&dbcon::schlagwort::Insert{
                        schlagwort: schlag.clone(),
                        beschreibung: "".to_owned()
                    })
                    .returning(crate::schema::schlagworte::dsl::id)
                    .get_result(conn)?;
                no_match_found(format!("Schlagwort {} was not found in database, inserted, please review, new Id = {}", 
                schlag.as_str(), id),
                app.clone());
                id
            }else{
                res[0]
            };
            schlagworte.push(dbcon::RelGesSchlagworte{
                gesetzesvorhaben: gesvh_db_id,
                schlagwort: schlag_id,
            });
        }
        diesel::insert_into(crate::schema::rel_ges_schlagworte::table)
            .values(&schlagworte)
            .execute(conn)?;
    }
    // insert eigenschaften
    if gesvh.eigenschaften.len() > 0 {
        let mut eigenschaften = vec![];
        for eig in gesvh.eigenschaften {
            let res: Vec<i32> = crate::schema::gesetzeseigenschaften::table
                .select(crate::schema::gesetzeseigenschaften::dsl::id)
                .filter(crate::schema::gesetzeseigenschaften::dsl::eigenschaft.eq(eig.clone()))
                .load::<i32>(conn)?;
            let eig_id = if res.is_empty(){
                // insert new eigenschaft and send email for review
                tracing::warn!("Eigenschaft {} not found in database, inserting and sending email for review", eig.as_str());
                let id : i32 = diesel::insert_into(crate::schema::gesetzeseigenschaften::table)
                    .values(&dbcon::gesetzeseigenschaft::Insert{
                        eigenschaft: eig.clone(),
                    })
                    .returning(crate::schema::gesetzeseigenschaften::dsl::id)
                    .get_result(conn)?;
                no_match_found(format!("Eigenschaft {} was not found in database, inserted, please review, new Id = {}", 
                eig.as_str(), id),
                app.clone());
                id
            }else{
                res[0]
            };
            eigenschaften.push(dbcon::RelGesEigenschaft{
                gesetzesvorhaben: gesvh_db_id,
                eigenschaft: eig_id,
            });
        }
        diesel::insert_into(crate::schema::rel_ges_eigenschaft::table)
            .values(&eigenschaften)
            .execute(conn)?;
    }
    // insert dokumente
    
    let response_data = match gesvh.dokumente.len() {
        0 => clapi::Gesetzesvorhaben{
            ext_id: Some(gen_id),
            ..Default::default()
        },
        _ => {
            let created_dok_ids = insert_dokumente(gesvh_db_id, gesvh.dokumente, conn, app)?;
            clapi::Gesetzesvorhaben{
                ext_id: Some(gen_id),
                dokumente: created_dok_ids.iter().map(|x| clapi::Dokument{
                        ext_id: Some(*x),
                        ..Default::default()
                    }).collect(),
                ..Default::default()
            }
        }
    };
    Ok(response_data)
}

/// Used to update gesetzesvorhaben with HTTP PUT
pub(crate) async fn put_gesvh(
    _app: Arc<AppState>,
    cupdate: clapi::CUPUpdate,
    gesvh_id: Uuid,
) -> std::result::Result<CUPResponse, LTZFError> {
    let update_data= cupdate.payload;

    if update_data.ext_id.is_some() && update_data.ext_id.unwrap() != gesvh_id {
        return Err(LTZFError::ParsingError(ParsingError::Internal(
            "Endpoint was called with a different Uuid than the one contained in the payload"
                .to_owned(),
        )));
    }
    todo!()
}

/// Used to create gesetzesvorhaben & associated data with HTTP POST
pub(crate) async fn post_gesvh(
    app: Arc<AppState>,
    cupdate: clapi::CUPUpdate,
) -> std::result::Result<CUPResponse, LTZFError> {
    let gesvh = cupdate.payload;
    let conn = app.pool.get().await.map_err(DatabaseError::from)?;
    if gesvh.ext_id.is_some() {
        return Err(LTZFError::WrongEndpoint(format!(
            "Used POST on Gesetzesvorhaben, but supplied ext_id {}",
            gesvh.ext_id.unwrap()
        )));
    }
    let gesvh_response = 
    conn.interact( move |conn| {
        let ta_res = 
            conn.transaction::<_, DatabaseError, _>(move |conn|{
                {
                    let cr_res = create_gesvh(gesvh, app, conn);
                    cr_res
                }
            });
        ta_res
        .map_err(LTZFError::from)
    }
    ).await
    .map_err(DatabaseError::from)??;
    let response = CUPResponse {
        msg_id: Uuid::now_v7(),
        timestamp: chrono::Utc::now(),
        responding_to: cupdate.msg_id,
        payload: clapi::Gesetzesvorhaben {
            ext_id: Some(gesvh_response.ext_id.unwrap()),
            ..Default::default()
        },
    };
    return Ok(response);
}
 
pub(crate) async fn get_gesvh(app: Arc<AppState>, gesvh_id: Uuid) -> Result<wsapi::WSResponse> {
    let conn = app.pool.get().await.map_err(DatabaseError::from)?;
    let gsv_res: dbcon::Gesetzesvorhaben = async_db!(conn, first, {
        crate::schema::gesetzesvorhaben::table
            .filter(crate::schema::gesetzesvorhaben::dsl::ext_id.eq(gesvh_id))
    });
    let dok_res: Vec<Uuid> = async_db!(conn, load, {
        crate::schema::dokumente::table
            .select(crate::schema::dokumente::dsl::ext_id)
            .filter(crate::schema::dokumente::dsl::gesetzesvorhaben.eq(gsv_res.id))
    });
    let eig_res: Vec<String> = async_db!(conn, load, {
        {
            crate::schema::rel_ges_eigenschaft::table
                .inner_join(crate::schema::gesetzeseigenschaften::table)
                .select(crate::schema::gesetzeseigenschaften::dsl::eigenschaft)
                .filter(crate::schema::rel_ges_eigenschaft::dsl::gesetzesvorhaben.eq(gsv_res.id))
        }
    });
    let init: Option<dbcon::Initiator> = if let Some(init) = gsv_res.initiat {
        Some(async_db!(conn, first, {
            {
                crate::schema::initiatoren::table
                    .filter(crate::schema::initiatoren::dsl::id.eq(gsv_res.initiat.unwrap()))
            }
        }))
    } else {
        None
    };
    let ff = if let Some(federf) = gsv_res.feder {
        let feder: dbcon::Ausschuss = async_db!(conn, first, {
            {
                crate::schema::ausschuesse::table
                    .filter(crate::schema::ausschuesse::dsl::id.eq(federf))
            }
        });
        Some(feder)
    } else {
        None
    };

    let status_rs: Vec<dbcon::Status> = async_db!(conn, load, {
        {
            use crate::schema::status as sm;
            use crate::schema::rel_ges_status as rm;
            rm::table
            .filter(rm::dsl::gesetzesvorhaben.eq(gsv_res.id))
            .inner_join(sm::table)
            .order(rm::dsl::datum.desc())
            .select((sm::dsl::id, sm::dsl::name, sm::dsl::parlament))
        }
    });
    todo!()
}

pub(crate) async fn get_gesvh_filtered(
    app: Arc<AppState>,
    params: GetGesvhQueryFilters,
) -> Result<wsapi::WSResponse> {
    let conn = app.pool.get().await.map_err(DatabaseError::from)?;

    todo!()
}

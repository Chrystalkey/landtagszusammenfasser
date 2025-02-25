/// Handles merging of two datasets.
/// in particular, stellungnahme & dokument are atomic.
/// station and vorgang are not in the sense that vorgang.stations and station.stellungnahmen are appendable and deletable.
/// This means the merge strategy is in general to:
/// 1. find a vorgang that is matching enough
///     a. if found exactly one, update the vorgang, see 2.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new vorgang, return
/// 2. if a., then update the vorgang properties
/// 3. for each station in the new vorgang, find a matching station
///     a. if found exactly one, update it, see 4.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new station & insert
/// 4. if a., then update station properties
/// 5. for each stellungnahme in the new station, find a matching stellungnahme
///    a. if found exactly one, replace it
///    b. if found more than one, send a message to the admins to select one
///    c. if found none, create a new stellungnahme & insert
use super::schema;
use crate::error::*;
use crate::utils;
use crate::{LTZFServer, Result};
use deadpool_diesel::postgres::Connection as AsyncConnection;
use diesel::prelude::*;
use diesel::Connection;
use openapi::models;
use std::collections::HashSet;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    /// this means the api ids are matching, which is a problem.
    ExactlyEqualMatch,
    NoMatch,
}

#[derive(QueryableByName, Debug, PartialEq, Eq, Hash, Clone)]
#[diesel(table_name=schema::vorgang)]
struct GSVHID {
    id: i32,
}
#[derive(QueryableByName, Debug, PartialEq, Eq, Hash, Clone)]
#[diesel(table_name=schema::station)]
struct STATID {
    id: i32,
}

/// this function determines what means "matching enough".
/// I propose:
/// 1. title match: if the titles are similar enough (to be determined)
/// 2. any existing station must match the parliamentary track of the incoming vorgang
///
pub async fn vorgang_merge_candidates(
    model: &models::Vorgang,
    connection: &AsyncConnection,
) -> Result<MergeState<(i32, models::Vorgang)>> {
    let apiid = model.api_id.clone();
    let x = connection
        .interact(move |conn| {
            schema::vorgang::table
                .filter(schema::vorgang::api_id.eq(apiid))
                .select(schema::vorgang::id)
                .first::<i32>(conn)
                .optional()
        })
        .await??;
    if x.is_some() {
        tracing::info!("Found exactly matching GSVH with api_id: {}", model.api_id);
        return Ok(MergeState::ExactlyEqualMatch);
    }

    let result = if let Some(ids) = model.ids.clone() {
        let query = "SELECT vorgang.id, titel FROM vorgang, gesetzestyp
        WHERE SIMILARITY(vorgang.titel, $1) > 0.8
        AND vorgang.typ = gesetzestyp.id
        AND gesetzestyp.api_key = $2
        OR EXISTS(
            SELECT 1 FROM rel_vorgang_id, identifikatortyp 
            WHERE rel_vorgang_id.vorgang_id = vorgang.id AND 
                    identifikatortyp.id = rel_vorgang_id.typ AND
                    identifikatortyp.api_key = $3 AND
                    rel_vorgang_id.identifikator = $4)";
        tracing::trace!("Executing Query: {}", query);
        let mut result: HashSet<GSVHID> = HashSet::new();

        for id in ids {
            let stat = model.stationen[0].parlament.to_string();
            let titel = model.titel.clone();
            let id_result = connection
                .interact(move |conn| {
                    diesel::sql_query(query)
                        .bind::<diesel::sql_types::Text, _>(stat)
                        .bind::<diesel::sql_types::Text, _>(titel)
                        .bind::<diesel::sql_types::Text, _>(id.typ.to_string())
                        .bind::<diesel::sql_types::Text, _>(id.id.clone())
                        .get_results::<GSVHID>(conn)
                })
                .await??;

            result.extend(id_result.iter().cloned());
        }
        result.drain().collect::<Vec<_>>()
    } else {
        // select where title is pretty equal and the stations belong to the same 
        let query = "SELECT vorgang.id, titel FROM vorgang, gesetzestyp
        WHERE SIMILARITY(vorgang.titel, $1) > 0.8
        AND vorgang.typ = gesetzestyp.id
        AND gesetzestyp.api_key = $2";
        tracing::trace!("Executing Query: {}", query);
        let titel = model.titel.clone();
        let typ = model.typ.to_string();
        let result = connection
            .interact(move |conn| {
                diesel::sql_query(query)
                    .bind::<diesel::sql_types::Text, _>(titel)
                    .bind::<diesel::sql_types::Text, _>(typ)
                    .get_results::<GSVHID>(conn)
            })
            .await??;
        result
    };

    tracing::debug!(
        "Found {} matches for GSVH with api_id: {}\n\n{:?}",
        result.len(),
        model.api_id,
        result
    );

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch((
            result[0].id,
            super::retrieve::vorgang_by_id(result[0].id, connection).await?,
        )),
        _ => {
            let mut asvec = vec![];
            for i in result {
                asvec.push((i.id, super::retrieve::vorgang_by_id(i.id, connection).await?));
            }
            MergeState::AmbiguousMatch(asvec)
        }
    })
}

/// Updates a GSVH based on similarity.
pub fn update_vorgang(
    model: &models::Vorgang,
    candidate: (i32, models::Vorgang),
    connection: &mut PgConnection,
) -> Result<()> {
    let db_id = candidate.0;
    diesel::update(schema::vorgang::table)
        .filter(schema::vorgang::id.eq(db_id))
        .set((
            schema::vorgang::verfaend.eq(model.verfassungsaendernd),
            schema::vorgang::wahlperiode.eq(model.wahlperiode as i32),
        ))
        .execute(connection)?;
    diesel::delete(schema::rel_vorgang_init::table)
        .filter(schema::rel_vorgang_init::vorgang_id.eq(db_id))
        .execute(connection)?;
    diesel::insert_into(schema::rel_vorgang_init::table)
        .values(
            model
                .initiatoren
                .iter()
                .map(|init| {
                    (
                        schema::rel_vorgang_init::vorgang_id.eq(db_id),
                        schema::rel_vorgang_init::initiator.eq(init.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .execute(connection)?;
    diesel::delete(schema::rel_vorgang_links::table)
        .filter(schema::rel_vorgang_links::vorgang_id.eq(db_id))
        .execute(connection)?;
    if let Some(links) = model.links.as_ref() {
        diesel::insert_into(schema::rel_vorgang_links::table)
            .values(
                links
                    .iter()
                    .map(|link| {
                        (
                            schema::rel_vorgang_links::vorgang_id.eq(db_id),
                            schema::rel_vorgang_links::link.eq(link.clone()),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(connection)?;
    }

    for station in model.stationen.iter() {
        let similarity_query = 
        if station.stellungnahmen.is_some() && station.stellungnahmen.as_ref().unwrap().len() > 0{
            format!(
                "SELECT station.id FROM station, stationstyp, parlament
            WHERE
            station.typ = stationstyp.id AND
            station.parl_id = parlament.id AND
            stationstyp.api_key = $1 AND
            parlament.api_key = $2 AND
            (FALSE
            OR EXISTS (
                SELECT 1 FROM dokument, rel_station_dokument WHERE 
                rel_station_dokument.stat_id = station.id AND
                rel_station_dokument.dok_id = dokument.id AND
                dokument.hash IN ({})
            )OR EXISTS (
                SELECT 1 FROM dokument, stellungnahme WHERE 
                stellungnahme.stat_id = station.id AND
                stellungnahme.dok_id = dokument.id AND
                dokument.hash IN ({})
            ))",
                station
                    .dokumente
                    .iter()
                    .map(|d| format!("'{}'", d.hash))
                    .collect::<Vec<_>>()
                    .join(","),
                station
                    .stellungnahmen
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|d| format!("'{}'", d.dokument.hash))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }else{
            format!(
                "SELECT station.id FROM station, stationstyp, parlament
            WHERE
            station.typ = stationstyp.id AND
            station.parl_id = parlament.id AND
            stationstyp.api_key = $1 AND
            parlament.api_key = $2 AND
            ( 
            OR EXISTS (
                SELECT 1 FROM dokument, rel_station_dokument WHERE 
                rel_station_dokument.stat_id = station.id AND
                rel_station_dokument.dok_id = dokument.id AND
                dokument.hash IN ({})
            ))", station
            .dokumente
            .iter()
            .map(|d| format!("'{}'", d.hash))
            .collect::<Vec<_>>()
            .join(",")
        )};
        let typ = station.typ.clone();
        let parl = station.parlament.clone();
        let result = diesel::sql_query(similarity_query.as_str())
            .bind::<diesel::sql_types::Text, _>(typ.to_string())
            .bind::<diesel::sql_types::Text, _>(parl.to_string())
            .bind::<diesel::sql_types::Integer, _>(db_id)
            .get_results::<STATID>(connection)?
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>();
        if result.is_empty() {
            super::insert::insert_station(station.clone(), db_id, connection)?;
        } else if result.len() == 1 {
            let stat_id = result[0];
            diesel::update(schema::station::table)
                .filter(schema::station::id.eq(stat_id))
                .set((
                    schema::station::trojanergefahr.eq(station.trojanergefahr.clone().unwrap_or(0) as i32),
                    schema::station::link.eq(station.link.clone()),
                    schema::station::zeitpunkt.eq(station.zeitpunkt),
                ))
                .execute(connection)?;
            // rep sw
            let schlagworte = station.schlagworte.clone().unwrap_or(vec![]);
            diesel::delete(schema::rel_station_schlagwort::table)
                .filter(schema::rel_station_schlagwort::stat_id.eq(stat_id))
                .execute(connection)?;
            diesel::insert_into(schema::schlagwort::table)
                .values(
                    schlagworte
                        .iter()
                        .map(|sw| schema::schlagwort::api_key.eq(sw.clone()))
                        .collect::<Vec<_>>(),
                )
                .on_conflict_do_nothing()
                .execute(connection)?;
            let sw_ids = schema::schlagwort::table
                .select(schema::schlagwort::id)
                .filter(schema::schlagwort::api_key.eq_any(schlagworte))
                .distinct()
                .get_results::<i32>(connection)?;
            diesel::insert_into(schema::rel_station_schlagwort::table)
                .values(
                    sw_ids
                        .iter()
                        .map(|id| {
                            (
                                schema::rel_station_schlagwort::stat_id.eq(stat_id),
                                schema::rel_station_schlagwort::sw_id.eq(*id),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .execute(connection)?;
            // rep doks
            for dokument in station.dokumente.iter() {
                let direct_equivalence = schema::dokument::table
                    .select(schema::dokument::id)
                    .filter(schema::dokument::hash.eq(dokument.hash.clone()))
                    .first::<i32>(connection)
                    .optional()?;

                // If the exact same document is already in the database, we can skip this step.
                if direct_equivalence.is_some() {
                    continue;
                }
                let id = super::insert::insert_dokument(dokument.clone(), connection)?;
                diesel::insert_into(schema::rel_station_dokument::table)
                    .values((
                        schema::rel_station_dokument::stat_id.eq(stat_id),
                        schema::rel_station_dokument::dok_id.eq(id),
                    ))
                    .execute(connection)?;
            }
            // rep stln
            if let Some(stellungnahmen) = station.stellungnahmen.clone() {
                for stellungnahme in stellungnahmen.iter() {
                    let direct_equivalence = schema::dokument::table
                        .select(schema::dokument::id)
                        .filter(schema::dokument::hash.eq(stellungnahme.dokument.hash.clone()))
                        .first::<i32>(connection)
                        .optional()?;
                    if direct_equivalence.is_some() {
                        continue;
                    }
                    let dok_id =
                        super::insert::insert_dokument(stellungnahme.dokument.clone(), connection)?;
                    diesel::insert_into(schema::stellungnahme::table)
                        .values((
                            schema::stellungnahme::meinung.eq(stellungnahme.meinung as i32),
                            schema::stellungnahme::dok_id.eq(dok_id),
                            schema::stellungnahme::stat_id.eq(stat_id),
                            schema::stellungnahme::lobbyreg_link.eq(stellungnahme
                                .lobbyregister_link
                                .clone()
                                .unwrap_or("".to_string())),
                        ))
                        .execute(connection)?;
                }
            }
        } else {
            tracing::warn!("Ambiguous matches for Station");
            todo!("Ambiguous matches for Station, send mail to admins")
        }
    }
    Ok(())
}

pub async fn run(model: &models::Vorgang, server: &LTZFServer) -> Result<()> {
    let connection = server.database.get().await?;
    tracing::debug!(
        "Looking for Merge Candidates for GSVH with api_id: {:?}",
        model.api_id
    );
    let candidates = vorgang_merge_candidates(model, &connection).await?;
    match candidates {
        MergeState::NoMatch => {
            tracing::info!(
                "No Merge Candidate found, Inserting GSVH with api_id: {:?}",
                model.api_id
            );
            let model = model.clone();
            //create GSVH
            connection
                .interact(move |conn| {
                    conn.transaction(|conn| super::insert::insert_vorgang(&model, conn))
                })
                .await??;
        }
        MergeState::OneMatch(one) => {
            tracing::info!(
                "Matching GSVH has api_id: {}, Updating with data from: {}",
                one.1.api_id,
                model.api_id
            );
            let model = model.clone();
            connection
                .interact(move |conn| conn.transaction(move |conn| update_vorgang(&model, one, conn)))
                .await??;
            //update GSVH
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!("Ambiguous matches for GSVH with api_id: {:?}", model.api_id);
            tracing::debug!("Ambiguous matches for GSVH:  {:?} \n\n {:?}", model, many);
            utils::send_email(
                "Ambiguous Match for Merge".to_string(), 
                "Fresh GSVH entered the database, producing ambiguous matches. The new GSVH is: \n\n {:?} \n\n The matches are: \n\n {:?}\n please provide guidance.".to_string(),
            server)?;
            return Err(DataValidationError::AmbiguousMatch {
                message: format!(
                    "Merge Candidates: {:?}",
                    many.iter().map(|e| e.1.api_id).collect::<Vec<_>>()
                ),
            }.into());
        }
        MergeState::ExactlyEqualMatch => {
            return Err(DataValidationError::DuplicateApiId{id: model.api_id}.into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod scenariotests{
    use std::collections::HashSet;
    use futures::FutureExt;
    use similar::ChangeTag;
    use std::panic::AssertUnwindSafe;

    use diesel::prelude::*;
    use deadpool_diesel::postgres::{Pool, Manager, Connection};
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use openapi::models::{self, VorgangGetHeaderParams, VorgangGetQueryParams};
    use serde::Deserialize;
    use crate::LTZFServer;

    #[allow(unused)]
    use tracing::{info, error, warn, debug};

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
    const DB_URL: &str = "postgres://mergeuser:mergepass@localhost:59512/mergecenter";

    #[allow(unused)]
    struct TestScenario<'obj>{
        name: &'obj str,
        context: Vec<models::Vorgang>,
        vorgang: models::Vorgang,
        result: Vec<models::Vorgang>,
        shouldfail: bool,
        server: LTZFServer,
        span: tracing::Span,
    }
    #[derive(Deserialize)]
    struct PTS{
        context: Vec<models::Vorgang>,
        vorgang: models::Vorgang,
        result: Vec<models::Vorgang>,
        #[serde(default = "default_bool")]
        shouldfail: bool
    }
    fn default_bool()->bool{ false }
    impl<'obj> TestScenario<'obj>{
        async fn new(path: &'obj std::path::Path, conn: &Connection) -> Self {
            let name = path.file_stem().unwrap().to_str().unwrap();
            info!("Creating Merge Test Scenario with name: {}", name);
            let span = tracing::span!(tracing::Level::INFO, "Mergetest", name = name);
            let query = format!("CREATE DATABASE testing_{} WITH OWNER mergeuser;", name);
            conn.interact(|conn| {
                diesel::sql_query(query)
                .execute(conn)
            }).await.unwrap().unwrap();

            let test_db_url = format!("postgres://mergeuser:mergepass@localhost:59512/testing_{}", name);
            let pts: PTS = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
            let server = LTZFServer {
                config: crate::Configuration{
                    ..Default::default()
                },
                database: Pool::builder(Manager::new(test_db_url,deadpool_diesel::Runtime::Tokio1)).build().unwrap(),
                mailer: None
            };
            let conn = server.database.get().await.unwrap();
            conn.interact(|conn| 
            conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await.unwrap().unwrap();
            info!("Migrations applied");
            for vorgang in pts.context.iter() {
                super::run(vorgang, &server).await.unwrap()
            }
            Self {
                name,
                context: pts.context,
                vorgang: pts.vorgang,
                result: pts.result,
                shouldfail: pts.shouldfail,
                span,
                server,
            }
        }
        async fn get_conn(&self) -> Connection {
            self.server.database.get().await.unwrap()
        }
        async fn push(&self) {
            info!("Running main Merge test");
            super::run(&self.vorgang, &self.server).await.unwrap();
        }
        async fn check(&self) {
            info!("Checking for Correctness");
            let paramock = VorgangGetQueryParams{
                ggtyp: None,
                initiator_contains_any: None, 
                limit: Some((self.result.len()*2) as i32),
                offset: None};
            let hparamock = VorgangGetHeaderParams{
                if_modified_since: None,
            };
            let db_vorgangs = crate::db::retrieve::vorgang_by_parameter(
                paramock, hparamock, &mut self.get_conn().await).await.unwrap();
            let mut set = HashSet::with_capacity(db_vorgangs.len());
            for thing in self.result.iter() {
                tracing::info!("Adding `{}` to result set", thing.api_id);
                set.insert(serde_json::to_string(thing).unwrap());
            }
            for thing in db_vorgangs.iter() {
                let serialized = serde_json::to_string(thing).unwrap();
                let result = set.remove(&serialized);
                if !result{
                    assert!(result, 
                        "Database value with api_id `{}` was not present in the result set, which contained: {:?}.\n\nDetails:\n{}", 
                    thing.api_id, 
                    self.result.iter().map(|e|e.api_id).collect::<Vec<uuid::Uuid>>(), display_set_strdiff(&serialized, set));
                }
            }
            assert!(set.is_empty(), "Values were expected, but not present in the result set: {:?}", set);
        }
        async fn run(self) {
            self.push().await;
            self.check().await;
        }
    }

    fn display_set_strdiff(s: &str, set: HashSet<String>) -> String {
        let mut prio = 0.;
        let mut pe_diff = None;
        for element in set.iter(){
            let diff = similar::TextDiff::from_chars(s, element);
            if prio < diff.ratio(){
                prio = diff.ratio();
                pe_diff = Some(diff);
            }
        }
        if let Some(diff) = pe_diff{
            let mut s = String::new();
            let mut diffiter = diff.iter_all_changes().filter(|x| x.tag() != ChangeTag::Equal);
            let mut current_sign = ChangeTag::Equal;
            while let Some(el) = diffiter.next(){
                let sign = match el.tag() {
                    ChangeTag::Equal => continue,
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+"
                };
                if el.tag() != current_sign{
                    s = format!("{}\n{:05}: {}| {}", s, el.old_index().unwrap_or(0), sign, el.value());
                    current_sign = el.tag();
                } else {
                    s = format!("{}{}", s, el.value());
                }
            }
            s
        }
        else{
            format!("Set is empty")
        }
    }
    
    #[tokio::test]
    async fn test_merge_scenarios() {
        // set up database connection and clear it
        info!("Setting up Test Database Connection");
        let pool = Pool::builder(Manager::new(DB_URL, deadpool_diesel::Runtime::Tokio1)).build().unwrap();
        let mut available = false;
        for i in 0..14 {
            let r = pool.get().await;
            match r {
                Ok(_) => {available = true;break;}
                Err(deadpool_diesel::PoolError::Backend(deadpool_diesel::Error::Connection(
                    ConnectionError::BadConnection(e)
                ))) => {
                    tracing::warn!("{}", e);
                },
                _ => {let _ = r.unwrap();}
            }
            let milliseconds = 2i32.pow(i) as u64;
            tracing::info!("DB Unavailable, Retrying in {} ms...", milliseconds);
            std::thread::sleep(std::time::Duration::from_millis(milliseconds));
        };
        if !available {
            panic!("Database unavailable");
        }

        let conn = pool.get().await.unwrap();
        for path in std::fs::read_dir("tests/testfiles").unwrap() {
            if let Ok(path) = path {
                info!("Executing Scenario: {}", path.path().display());
                let ptb = path.path();
                let name = ptb.file_stem().unwrap().to_str().unwrap();

                let mut shouldfail = false;
                let result = AssertUnwindSafe(async {
                    let scenario = TestScenario::new(&ptb, &conn).await;
                    shouldfail = scenario.shouldfail;
                    scenario.run().await
                }
                ).catch_unwind().await;

                let query = format!("DROP DATABASE testing_{}", name);
                conn.interact(move |conn|{
                    diesel::sql_query(query)
                    .execute(conn)
                }).await.unwrap().unwrap();
                if result.is_ok() == shouldfail {
                    assert!(false, "The Scenario did not behave as expected: {}", 
                    if shouldfail{"Succeeded, but should fail"}else{"Failed but should succeed"}
                    );
                }
            }else{
                error!("Error: {:?}", path.unwrap_err())
            }
        }

    }
}
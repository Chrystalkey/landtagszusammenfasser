#![allow(unused)]
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

use crate::{LTZFServer, Result};
use openapi::models;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    NoMatch,
}

/// this function determines what means "matching enough".
/// I propose:
/// 1. title match: if the titles are similar enough (to be determined)
/// 2. any existing station must match the parliamentary track of the incoming vorgang
///
pub async fn vorgang_merge_candidates(
    model: &models::Vorgang,
    executor: impl sqlx::PgExecutor<'_>,
    srv: &LTZFServer,
) -> Result<MergeState<i32>> {
    let obj = "merged Vorgang";
    let ident_t: Vec<_> = model.ids.as_ref().unwrap_or(&vec![]).iter().map(|x|x.id.clone()).collect();
    let identt_t: Vec<_> = model.ids.as_ref().unwrap_or(&vec![]).iter().map(|x| srv.guard_ts(x.typ, model.api_id, obj).unwrap()).collect();
    let result = sqlx::query!(
        "WITH db_id_table AS (
SELECT rel_vorgang_ident.vorgang_id as vgid, identifikator as ident, vg_ident_typ.api_key as idt_str
FROM vg_ident_typ, rel_vorgang_ident 
WHERE vg_ident_typ.id = rel_vorgang_ident.typ)

SELECT DISTINCT(vorgang.id), vorgang.api_id FROM vorgang
JOIN vorgangstyp ON vorgang.typ = vorgangstyp.id
WHERE
vorgang.api_id = $1 OR
(
vorgang.wahlperiode = $4 AND 
vorgangstyp.api_key = $5 AND
    EXISTS (SELECT * FROM UNNEST($2::text[], $3::text[]) as eingabe(ident, typ), db_id_table WHERE 
        db_id_table.vgid = vorgang.id AND
        eingabe.ident = db_id_table.ident AND
        eingabe.typ = db_id_table.idt_str
    )
);", model.api_id, &ident_t[..], &identt_t[..], model.wahlperiode as i32, srv.guard_ts(model.typ, model.api_id, obj)?)
    .fetch_all(executor).await?;

    tracing::debug!("Found {} matches for Vorgang with api_id: {}",result.len(),model.api_id);

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch(result[0].id),
        _ => {
            tracing::warn!("Warning: Mehrere Vorgänge gefunden, die als Kandidaten für Merge infrage kommen für den Vorgang `{}`:\n{:?}", 
            model.api_id, result.iter().map(|r|r.api_id).collect::<Vec<_>>());
            MergeState::AmbiguousMatch(
                result.iter().map(|x|x.id).collect()
            )
        }
    })
}
pub async fn station_merge_candidates(model: &models::Station, executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer)-> Result<MergeState<i32>> {
    todo!()
}
pub async fn dokument_merge_candidates(model: &models::Station, executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer)->Result<MergeState<i32>>{
    todo!()
}

pub async fn execute_merge_dokument (
    model: &models::Dokument,
    candidate: i32,
    executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    todo!()
}
pub async fn execute_merge_station (
    model: &models::Station,
    candidate: i32,
    executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    todo!()
}
pub async fn execute_merge_vorgang (
    model: &models::Vorgang,
    candidate: i32,
    executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    todo!()
}

pub async fn run_integration(model: &models::Vorgang, server: &LTZFServer) -> Result<()> {
    let mut tx = server.sqlx_db.begin().await?;
    tracing::debug!(
        "Looking for Merge Candidates for Vorgang with api_id: {:?}",
        model.api_id);
    let candidates = vorgang_merge_candidates(model, &mut *tx, server).await?;
    match candidates {
        MergeState::NoMatch => {
            tracing::info!(
                "No Merge Candidate found, Inserting Complete Vorgang with api_id: {:?}",
                model.api_id
            );
            let model = model.clone();
            super::insert::insert_vorgang(&model, &mut tx, server).await?;
        }
        MergeState::OneMatch(one) => {
            let api_id = sqlx::query!("SELECT api_id FROM vorgang WHERE id=$1", one)
            .map(|r|r.api_id).fetch_one(&mut *tx).await?;
            tracing::info!(
                "Matching Vorgang in the DB has api_id: {}, Updating with data from: {}",
                api_id,
                model.api_id
            );
            let model = model.clone();
            execute_merge_vorgang(&model, one, &mut *tx, server).await?;
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!("Ambiguous matches for Vorgang with api_id: {:?}", model.api_id);
            tracing::debug!("Details:  {:?} \n\n {:?}", model, many);
            unimplemented!("Notify Admins via $WAY");
        }
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod scenariotests{
    use std::collections::HashSet;
    use futures::FutureExt;
    use similar::ChangeTag;
    use std::panic::AssertUnwindSafe;

    use openapi::models::{self, VorgangGetHeaderParams, VorgangGetQueryParams};
    use serde::Deserialize;
    use crate::LTZFServer;

    #[allow(unused)]
    use tracing::{info, error, warn, debug};

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
        async fn new(path: &'obj std::path::Path, server: &LTZFServer) -> Self {
            let name = path.file_stem().unwrap().to_str().unwrap();
            info!("Creating Merge Test Scenario with name: {}", name);
            let span = tracing::span!(tracing::Level::INFO, "Mergetest", name = name);
            let query = format!("CREATE DATABASE testing_{} WITH OWNER mergeuser;", name);
            todo!();

            let test_db_url = format!("postgres://mergeuser:mergepass@localhost:59512/testing_{}", name);
            let pts: PTS = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
            let server = LTZFServer {
                config: crate::Configuration{
                    ..Default::default()
                },
                mailer: None,
                sqlx_db: sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&test_db_url).await.unwrap()
            };
            for vorgang in pts.context.iter() {
                super::run_integration(vorgang, &server).await.unwrap()
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
        async fn push(&self) {
            info!("Running main Merge test");
            super::run_integration(&self.vorgang, &self.server).await.unwrap();
        }
        async fn check(&self) {
            info!("Checking for Correctness");
            let paramock = VorgangGetQueryParams{
                vgtyp: None,
                wp: None,
                initiator_contains_any: None, 
                limit: Some((self.result.len()*2) as i32),
                offset: None};
            let hparamock = VorgangGetHeaderParams{
                if_modified_since: None,
            };
            let mut tx = self.server.sqlx_db.begin().await.unwrap();
            let db_vorgangs = crate::db::retrieve::vorgang_by_parameter(
                paramock, hparamock, &mut tx).await.unwrap();
            tx.commit().await.unwrap();
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

        for path in std::fs::read_dir("tests/testfiles").unwrap() {
            if let Ok(path) = path {
                info!("Executing Scenario: {}", path.path().display());
                let ptb = path.path();
                let name = ptb.file_stem().unwrap().to_str().unwrap();

                let mut shouldfail = false;
                let result = AssertUnwindSafe(async {
                    let scenario = TestScenario::new(&ptb, todo!()).await;
                    shouldfail = scenario.shouldfail;
                    scenario.run().await
                }
                ).catch_unwind().await;
                todo!();
                // let query = format!("DROP DATABASE testing_{}", name);
                // conn.interact(move |conn|{
                //     diesel::sql_query(query)
                //     .execute(conn)
                // }).await.unwrap().unwrap();
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
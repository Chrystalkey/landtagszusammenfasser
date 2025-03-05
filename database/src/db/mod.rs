pub mod insert;
pub mod retrieve;
pub mod merge;
pub mod delete;

mod scenariotest{
    #![cfg(test)]
    use std::collections::HashSet;
    use futures::FutureExt;
    use similar::ChangeTag;
    use std::panic::AssertUnwindSafe;
    use crate::LTZFServer;

    use openapi::models::{self, VorgangGetHeaderParams, VorgangGetQueryParams};
    use serde::Deserialize;

    #[allow(unused)]
    use tracing::{info, error, warn, debug};

    fn xor(one: bool, two: bool) -> bool{
        return (one &&two) || (!one && !two);
    }
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
    struct PTS {
        context: Vec<models::Vorgang>,
        vorgang: models::Vorgang,
        result: Vec<models::Vorgang>,
        #[serde(default = "default_bool")]
        shouldfail: bool,
    }
    fn default_bool()->bool{ false }
    impl<'obj> TestScenario<'obj>{
        async fn new(path: &'obj std::path::Path, server: &LTZFServer) -> Self {
            let name = path.file_stem().unwrap().to_str().unwrap();
            info!("Creating Merge Test Scenario with name: {}", name);
            let span = tracing::span!(tracing::Level::INFO, "Mergetest", name = name);
            let query = format!("CREATE DATABASE testing_{} WITH OWNER 'ltzf-user';", name);
            sqlx::query(&query).execute(&server.sqlx_db).await.unwrap();
            let test_db_url = std::env::var("DATABASE_URL").unwrap()
                .replace("5432/ltzf", &format!("5432/testing_{}", name));
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
            sqlx::migrate!().run(&server.sqlx_db).await.unwrap();
            for vorgang in pts.context.iter() {
                crate::db::merge::vorgang::run_integration(vorgang, &server).await.unwrap()
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
            crate::db::merge::vorgang::run_integration(&self.vorgang, &self.server).await.unwrap();
        }
        async fn check(&self) {
            info!("Checking for Correctness");
            let paramock = VorgangGetQueryParams{
                vgtyp: None,
                wp: None,
                initiator_contains_any: None, 
                limit: None,
                offset: None};
            let hparamock = VorgangGetHeaderParams{
                if_modified_since: None,
            };
            let mut tx = self.server.sqlx_db.begin().await.unwrap();
            let db_vorgangs = crate::db::retrieve::vorgang_by_parameter(
                paramock, hparamock, &mut tx).await.unwrap();
                
            tx.rollback().await.unwrap();
            for expected in self.result.iter() {
                let mut found = false;
                for db_out in db_vorgangs.iter() {
                    if db_out == expected {
                        found = true;
                        break;
                    }else if xor(db_out.api_id == expected.api_id, self.shouldfail) {
                        std::fs::write(format!("tests/{}_dumpa.json", self.name), 
                        dump_objects(&expected, &db_out)).unwrap();
                        assert!(false, "Differing object have the same api id: `{}`. Difference:\n{}",
                        db_out.api_id, display_strdiff(&serde_json::to_string_pretty(db_out).unwrap(), &serde_json::to_string_pretty(expected).unwrap())
                        );
                    }
                    
                }
                if xor(!found, self.shouldfail) {
                    std::fs::write(format!("tests/{}_dump.json", self.name), 
                    serde_json::to_string_pretty(expected).unwrap()).unwrap();
                }
                assert!(found, 
                    "Expected to find Vorgang with api_id `{}`, but was not present in the output set, which contained: {:?}.\n\nDetails(Output Set):\n{:#?}", 
                expected.api_id, 
                self.result.iter().map(|e|e.api_id).collect::<Vec<uuid::Uuid>>(),
                db_vorgangs.iter().map(|v|
                {println!("{}", serde_json::to_string_pretty(v).unwrap());""})
                .collect::<Vec<_>>()
                );
            }
            
            assert!(self.result.len()==db_vorgangs.len(), 
            "Mismatch between the length of the expected set and the output set: {} (e) vs {} (o)\nOutput Set: {:#?}", 
            self.result.len(), db_vorgangs.len(), db_vorgangs);

        }
        async fn run(self) {
            self.push().await;
            self.check().await;
        }
    }
    fn display_strdiff(s1: &str, s2: &str) -> String{
        let diff = similar::TextDiff::from_chars(s1, s2);
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
    #[allow(unused)]
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
        let test_db_url = std::env::var("DATABASE_URL").unwrap();
        let master_server = LTZFServer{
            config: crate::Configuration{
                ..Default::default()
            },
            mailer: None,
            sqlx_db: sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&test_db_url).await.unwrap()
        };

        for path in std::fs::read_dir("tests/testfiles").unwrap() {
            if let Ok(path) = path {
                info!("Executing Scenario: {}", path.path().display());
                let ptb = path.path();
                let name = ptb.file_stem().unwrap().to_str().unwrap();

                let mut shouldfail = false;
                let scenario = TestScenario::new(&ptb, &master_server).await;
                let result = AssertUnwindSafe(async {
                    shouldfail = scenario.shouldfail;
                    scenario.run().await
                }
                ).catch_unwind().await;
                
                if result.is_ok() == shouldfail {
                    assert!(false, "The Scenario {} did not behave as expected: {}", 
                    name,
                    if shouldfail{"Succeeded, but should fail"}else{"Failed but should succeed"}
                    );
                }else{
                    let query = format!("DROP DATABASE testing_{}", name);
                    sqlx::query(&query)
                    .execute(&master_server.sqlx_db).await.unwrap();
                }
            }else{
                error!("Error: {:?}", path.unwrap_err())
            }
        }
    }
    fn dump_objects<T: serde::Serialize, S: serde::Serialize>(expected: &T, actual: &S) -> String {
        format!("{{ \"expected-object\" : {},\n\"actual-object\" : {}}}", 
        serde_json::to_string_pretty(expected).unwrap(), serde_json::to_string_pretty(actual).unwrap()
        )
    }
}
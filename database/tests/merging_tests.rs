#![cfg(test)]

use diesel::prelude::*;
use deadpool_diesel::postgres::{Pool, Manager, Connection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use openapi::models;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

async fn build_pool() -> Pool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let manager = Manager::new(db_url.as_str(), deadpool_diesel::Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();
    return pool;
}
struct TestScenario{
    context: Vec<models::Gesetzesvorhaben>,
    gsvh: models::Gesetzesvorhaben,
    result: Vec<models::Gesetzesvorhaben>
}
impl TestScenario{
    async fn setup(&self, conn: &Connection, name: &str){
        let query = format!("CREATE DATABASE testing_{};\\c testing_{}", name, name);
        conn.interact(|conn|{
            diesel::sql_query(query)
            .execute(conn)
        }).await.unwrap().unwrap();
        conn.interact(|conn| 
        conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await.unwrap().unwrap()
        for gsvh in self.context{
            conn.interact(|conn|{
                db::merge::run()
            })
        }
    }
    async fn push(&self, conn: &Connection){}
    async fn check(&self, conn: &Connection){}
    async fn run(&self, conn: &Connection, name: &str){
        self.setup(conn, name).await;
        self.push(conn).await;
        self.check(conn).await;
    }
}

#[tokio::test]
async fn test_merge_scenarios() {
    // set up database connection and clear it
    let pool = build_pool().await;
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
    // for each scenario in the testfiles folder

}
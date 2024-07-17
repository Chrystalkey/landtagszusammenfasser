use crate::{error::DatabaseError, infra::db::schema::gesetzesvorhaben};
use diesel::{Insertable, Queryable, RunQueryDsl, Selectable};
use ulid::Ulid;
use uuid::Uuid;
use crate::util::from_ulid;

use super::EntityDBInteraction;

#[derive(Queryable, Insertable, Selectable, Debug, Default, Clone)]
#[diesel(table_name=gesetzesvorhaben)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Gesetzesvorhaben {
    id: Uuid,
    titel: String,
    off_titel: String,
    url_gesblatt: Option<String>,
    id_gesblatt: Option<String>,
    verfassungsaendernd: bool,
    trojaner: Option<bool>,
    federfuehrung: i32,
    initiator: i32,
}

impl Gesetzesvorhaben {
    pub fn new(verf_aend: bool, title: String) -> Self {
        Self {
            id: from_ulid(Ulid::new()),
            ..Default::default()
        }
    }
    pub fn with_off_title(mut self, off_title: String) -> Self {
        self.off_titel = off_title;
        self
    }
    pub fn with_url_gesblatt(mut self, url_gesblatt: Option<String>) -> Self {
        self.url_gesblatt = url_gesblatt;
        self
    }
    pub fn with_id_gesblatt(mut self, id_gesblatt: Option<String>) -> Self {
        self.id_gesblatt = id_gesblatt;
        self
    }
    pub fn with_trojaner(mut self, trojaner: Option<bool>) -> Self {
        self.trojaner = trojaner;
        self
    }
    pub fn with_federfuehrung(mut self, federfuehrung: i32) -> Self {
        self.federfuehrung = federfuehrung;
        self
    }
    pub fn with_initiator(mut self, initiator: i32) -> Self {
        self.initiator = initiator;
        self
    }
}

impl EntityDBInteraction<Uuid> for Gesetzesvorhaben {
    async fn insert(&self, pool: &deadpool_diesel::postgres::Pool) -> Result<(), DatabaseError> {
        let conn = pool.get().await?;
        let object = self.clone();
        let _res = conn
            .interact(|conn| {
                diesel::insert_into(gesetzesvorhaben::table)
                    .values(object)
                    .get_result::<Self>(conn)
            })
            .await??;
        Ok(())
    }

    async fn update(&self, pool: &deadpool_diesel::postgres::Pool) -> Result<(), DatabaseError> {
        todo!()
    }

    async fn delete(&self, pool: &deadpool_diesel::postgres::Pool) -> Result<(), DatabaseError> {
        todo!()
    }
    

    async fn get_all(
        &self,
        pool: &deadpool_diesel::postgres::Pool,
    ) -> Result<Vec<Self>, DatabaseError> {
        todo!()
    }
    
    async fn get_by_id(
        &self,
        id: Uuid,
        pool: &deadpool_diesel::postgres::Pool,
    ) -> Result<Self, DatabaseError> {
        todo!()
    }
}

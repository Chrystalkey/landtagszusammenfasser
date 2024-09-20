use crate::async_db;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DatabaseError;

#[derive(Debug, Serialize)]
pub struct WSResponse {
    pub id: Uuid,
    pub payload: WSPayload,
}

#[derive(Debug, Serialize)]
pub enum WSPayload {
    Gesetzesvorhaben(Vec<Gesetzesvorhaben>),
    Dokumente(Vec<Dokument>),
    Stationen(Vec<Station>),
    Ausschuesse(Vec<Ausschuss>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CUPUpdate {
    pub msg_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: Gesetzesvorhaben,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum FatOption<DataType, IDType>
where
    IDType: Copy,
{
    #[serde(untagged)]
    Data(DataType),
    #[serde(untagged)]
    Id(IDType),
}
impl<D, I> FatOption<D, I>
where
    I: Copy,
{
    #[allow(dead_code)]
    pub fn unwrap_id(&self) -> std::result::Result<I, &str> {
        match self {
            FatOption::Id(id) => Ok(*id),
            _ => Err("Tried to unwrap a FatOption::Data as an ID"),
        }
    }
    #[allow(dead_code)]
    pub fn unwrap_data(&self) -> std::result::Result<&D, &str> {
        match self {
            FatOption::Data(data) => Ok(data),
            _ => Err("Tried to unwrap a FatOption::ID as Data"),
        }
    }
}

/// These are the response structures. A CUPResponse is sent to the collector to notify it of the state of data after the update.
#[derive(Serialize, Deserialize, Debug)]
pub struct CUPResponse {
    pub msg_id: Uuid,
    pub responding_to: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: Gesetzesvorhaben,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Ausschuss {
    pub name: String,
    pub parlament: [char; 2],
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Station {
    pub status: String,
    pub datum: DateTime<Utc>,
    pub url: Option<String>,
    pub parlament: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub schlagworte: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub dokumente: Vec<FatOption<Dokument, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ausschuss: Option<FatOption<Ausschuss, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub meinungstenzdenz: Option<i32>,
}
impl Station{
    async fn create_from(thing: super::db::connection::Station)
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Gesetzesvorhaben {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_id: Option<Uuid>,
    pub titel: String,
    pub verfassungsaendernd: bool,
    pub trojaner: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub federfuehrung: Option<FatOption<Ausschuss, i32>>,
    pub initiator: String,
    pub typ: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub notes: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub stationen: Vec<FatOption<Station, i32>>,
}
impl Gesetzesvorhaben {
    pub async fn construct_from(
        data: super::db::connection::Gesetzesvorhaben,
        conn: deadpool_diesel::postgres::Connection,
    ) -> Result<Self, DatabaseError> {
        let stationen: Vec<i32> = async_db!(conn, load, {
            crate::schema::station::table
                .select(crate::schema::station::dsl::id)
                .filter(crate::schema::station::dsl::gesetzesvorhaben.eq(data.id))
        });
        Ok(Self {
            api_id: Some(data.api_id),
            titel: data.titel,
            verfassungsaendernd: data.verfassungsaendernd,
            trojaner: data.trojaner,
            federfuehrung: data.federf.map(|x| FatOption::Id(x)),
            initiator: data.initiator,
            typ: async_db!(conn, first, {
                crate::schema::gesetzestyp::table
                    .select(crate::schema::gesetzestyp::dsl::value)
                    .filter(crate::schema::gesetzestyp::dsl::id.eq(data.typ))
            }),
            links: async_db!(conn, load, {
                crate::schema::further_links::table
                    .select(crate::schema::further_links::dsl::link)
                    .filter(crate::schema::further_links::dsl::gesetzesvorhaben.eq(data.id))
            }),
            stationen: stationen.iter().map(|x| FatOption::Id(*x)).collect(),
            notes: async_db!(conn, load, {
                crate::schema::further_notes::table
                    .select(crate::schema::further_notes::dsl::notes)
                    .filter(crate::schema::further_notes::dsl::gesetzesvorhaben.eq(data.id))
            }),
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Dokument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_id: Option<Uuid>,
    pub identifikator: String,
    pub titel: String,
    pub hash: String,
    pub zsmfassung: String,
    pub typ: String,
    pub url: String,
    pub autoren: Vec<(String, String)>, // name and organisation

    pub letzter_zugriff: chrono::DateTime<Utc>,
}

impl Dokument {
    #[allow(dead_code)]
    pub async fn construct_from(
        dbdok: crate::infra::db::connection::Dokument,
        conn: deadpool_diesel::postgres::Connection,
    ) -> std::result::Result<Self, DatabaseError> {
        let doktyp: String = async_db!(conn, first, {
            crate::schema::dokumententyp::table
                .select(crate::schema::dokumententyp::dsl::value)
                .filter(crate::schema::dokumententyp::dsl::id.eq(dbdok.doktyp))
        });

        let autoren: Vec<(String, String)> = async_db!(conn, load, {
            crate::schema::autor::table
                .select((
                    crate::schema::autor::dsl::name,
                    crate::schema::autor::dsl::organisation,
                ))
                .filter(crate::schema::autor::dsl::id.eq(dbdok.id))
        });

        Ok(Self {
            api_id: Some(dbdok.api_id),
            identifikator: dbdok.identifikator,
            titel: dbdok.titel,
            hash: dbdok.hash,
            zsmfassung: dbdok.zsmfassung,
            typ: doktyp,
            url: dbdok.url,
            autoren: autoren,
            letzter_zugriff: DateTime::from_naive_utc_and_offset(dbdok.last_access, Utc),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_json_form() {
        let date : DateTime<Utc>= DateTime::parse_from_rfc3339("2024-09-19T12:12:20Z").unwrap().into();
        let data = Gesetzesvorhaben {
            titel: "Test".to_string(),
            verfassungsaendernd: false,
            trojaner: false,
            typ: "Einspruchsgesetz".to_string(),
            initiator: "BAMF".to_string(),
            stationen: vec![FatOption::Data(Station {
                status: "Entwurf: Eckpunktepapier".to_string(),
                datum: date,
                parlament: "BY".to_string(),
                url: Some("https://example.com".to_string()),
                ..Default::default()
            })],
            ..Default::default()
        };
        eprintln!("Serialized data: {}", serde_json::to_string(&data).unwrap());
        let json: &str = r#"{
            "titel": "Test",
            "trojaner": false,
            "initiator": "BAMF",
            "typ": "Einspruchsgesetz",
            "verfassungsaendernd": false,
            "stationen" : [
                {
                    "status": "Entwurf: Eckpunktepapier",
                    "datum" : "2024-09-19T12:12:20Z",
                    "url": "https://example.com",
                    "parlament": "BY"
                }
            ]
            }"#;
        let parsed_data: super::Gesetzesvorhaben = serde_json::from_str(json).unwrap();
        assert_eq!(parsed_data, data);
    }
}

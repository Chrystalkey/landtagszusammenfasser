use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// these are Collector-Updateable Structures (including associated data)
/// All other structures can only be updated indirectly.
/// One example could be: There is a status update on some Law. The collector
/// remembered the uuid of the `Gesetzesvorhaben` and consequently sends:
/// ```
/// { "msg_id": "something"
///   "timestamp": "2024-08-26T12:10:00",
///   "payload" : [
///         {
///             "uuid" : "abc123",
///             "data" : {..., "status": [{"name": "Eingegangen im Bundestag", ...}]}
///         }
///     ]
/// }
/// ```
/// The database checks for the uuid.
/// if found updates the status, returns
/// ```
/// {
///     "msg_id": "something else",
///     "responding_to": "something",
///     "timestamp": "2024-08-26T12:10:30",
///     "payload": [{"abc123": "Updated"}]
/// }
/// ```
/// If not found, returns CouldNotMatch instead of Updated, and the collector removes the Uuid from it's known
/// ressources

/// Sent from the collectors to the database
/// contains a collector-generated uuid, a timestamp and a list
/// of potentially new data.
/// If the collector does not know wether a specific piece of info is new or exists
/// the payload id is set to None (null) and the database checks entries for matches
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CUPUpdate {
    pub msg_id: Uuid,
    pub timestamp: chrono::DateTime<Utc>,
    pub payload: CUPPayload,
}

/// This is the enumeration of all top-level independent updateable entities.
/// Currently only Gesetzesvorhaben are supported, but in the future other data structures
/// may be added.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[non_exhaustive]
pub enum CUPPayload {
    GesVH(Gesetzesvorhaben),
    Dokument(Dokument),
    // TODO: Abstimmungen, Ausschusssitzungen, TOPs, etc.
}

/// These are the response structures. A CUPResponse is sent to the collector to notify it of the state of data after the update.
#[derive(Serialize, Deserialize, Debug)]
pub struct CUPResponse {
    pub msg_id: Uuid,
    pub responding_to: Uuid,
    pub timestamp: chrono::DateTime<Utc>,
    pub payload: CUPResponsePayload,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CUPResponsePayload {
    pub data: CUPPayload,
    pub state: CUPRessourceState,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum CUPRessourceState {
    Created,
    Updated,
    Exists,
    CouldNotMatch,
    Error(String),
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cupupdate() {
        let empty_gesvh = Gesetzesvorhaben {
            ext_id: Some(Uuid::now_v7()),
            titel: Some("Test".to_string()),
            off_titel: Some("Test".to_string()),
            dokumente: vec![Dokument {
                letzter_zugriff: Utc::now(),
                pfad: Some("https://example.com".to_owned()),
                typ: "Beschlussempfehlung".to_owned(),
                file_type: "pdf".to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let transfer_structure = CUPUpdate {
            msg_id: Uuid::now_v7(),
            timestamp: Utc::now(),
            payload: CUPPayload::GesVH(empty_gesvh),
        };
        let serialized = serde_json::to_string_pretty(&transfer_structure).unwrap();
        println!("serialized_transfer: {}", serialized);
        let deserialized_transfer: CUPUpdate = serde_json::de::from_str(&serialized).unwrap();
        assert_eq!(transfer_structure, deserialized_transfer);
    }
}
use crate::infra::db::connection as dbcon;
use crate::error::*;
pub trait DatabaseInteraction {
    #[allow(async_fn_in_trait)]
    async fn fetch_id(&self, conn: &mut deadpool_diesel::postgres::Connection) -> Result<i32>;
}

async fn fetch_parlament_id(
    parlament: [char; 2],
    conn: &mut deadpool_diesel::postgres::Connection,
) -> Result<i32> {
    let query = dbcon::parlamente::Update {
        name: None,
        kurzname: Some(parlament.iter().collect()),
    };
    let result = dbcon::parlamente::select_matching(conn, query)
    .await
    .map_err(DatabaseError::from)?;
    if result.len() == 1 {
        Ok(result[0].id)
    } else if result.len() > 1 {
        tracing::warn!("Ambiguous Match for Parlament: {:?}, using {:?} as query", result, parlament);  
        Err(RetrievalError::AmbiguousMatch(format!("Parlamente: {:?}", result)).into())
    }else{
        Err(RetrievalError::NoMatch.into())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Ausschuss {
    pub name: String,
    pub parlament: [char; 2],
}
impl DatabaseInteraction for Ausschuss {
    async fn fetch_id(&self, conn: &mut deadpool_diesel::postgres::Connection) -> Result<i32> {
        let query = dbcon::ausschuesse::Update {
            name: Some(self.name.clone()),
            parlament: Some(Some(fetch_parlament_id(self.parlament, conn).await?)),
        };
        let result = dbcon::ausschuesse::select_matching(conn, query).await.map_err(DatabaseError::from)?;
        if result.len() == 1 {
            Ok(result[0].id)
        } else if result.len() > 1 {
            tracing::warn!("Ambiguous Match for Ausschuessse: {:?}, using {:?} as query", result, self);  
            Err(RetrievalError::AmbiguousMatch(format!("Ausschuesse: {:?}", result)).into())
        }else{
            Err(RetrievalError::NoMatch.into())
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Initiator {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub organisation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub url: Option<String>,
}

impl DatabaseInteraction for Initiator {
    async fn fetch_id(&self, conn: &mut deadpool_diesel::postgres::Connection) -> Result<i32> {
        let query = dbcon::initiatoren::Update {
            name: Some(self.name.clone()),
            organisation: self.organisation.clone(),
            url: self.url.clone(),
        };
        let result = dbcon::initiatoren::select_matching(conn, query).await.map_err(DatabaseError::from)?;
        if result.len() == 1 {
            Ok(result[0].id)
        } else if result.len() > 1 {
            tracing::warn!("Ambiguous Match for Initiatoren: {:?}, using {:?} as query", result, self);  
            Err(RetrievalError::AmbiguousMatch(format!("Initiatoren: {:?}", result)).into())
        }else{
            Err(RetrievalError::NoMatch.into())
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Status {
    pub name: String,
    pub parlament: [char; 2],
}

impl DatabaseInteraction for Status {
    async fn fetch_id(&self, conn: &mut deadpool_diesel::postgres::Connection) -> Result<i32> {
        let query = dbcon::status::Update {
            name: Some(self.name.clone()),
            parlament: Some(Some(fetch_parlament_id(self.parlament, conn).await?)),
        };
        let result = dbcon::status::select_matching(conn, query).await.map_err(DatabaseError::from)?;
        if result.len() == 1 {
            Ok(result[0].id)
        } else if result.len() > 1 {
            tracing::warn!("Ambiguous Match for Status: {:?}, using {:?} as query", result, self);
            Err(RetrievalError::AmbiguousMatch(format!("Status: {:?}", result)).into())
        }else{
            Err(RetrievalError::NoMatch.into())
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Gesetzesvorhaben {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ext_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub titel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub off_titel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub url_gesblatt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub id_gesblatt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub verfassungsaendernd: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub trojaner: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub federfuehrung: Option<Ausschuss>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub initiator: Option<Initiator>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub dokumente: Vec<Dokument>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub status: Vec<Status>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub schlagworte: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub eigenschaften: Vec<String>,
}

impl DatabaseInteraction for Gesetzesvorhaben {
    async fn fetch_id(&self, conn: &mut deadpool_diesel::postgres::Connection) -> Result<i32> {
        let feder = if let Some(feder) = &self.federfuehrung{
            Some(feder.fetch_id(conn).await?)
        }else{None};
        let init = if let Some(init) = &self.initiator{
            Some(init.fetch_id(conn).await?)
        }else{None};
        let query = dbcon::gesetzesvorhaben::Update {
            ext_id: self.ext_id,
            titel: self.titel.clone(),
            off_titel: self.off_titel.clone(),
            url_gesblatt: Some(self.url_gesblatt.clone()),
            id_gesblatt: Some(self.id_gesblatt.clone()),
            verfassungsaendernd: self.verfassungsaendernd.clone(),
            trojaner: Some(self.trojaner.clone()),
            feder: Some(feder),
            initiat: Some(init),
        };
        let result = dbcon::gesetzesvorhaben::select_matching(conn, query).await.map_err(DatabaseError::from)?;
        if result.len() == 1 {
            Ok(result[0].id)
        } else if result.len() > 1 {
            tracing::warn!("Ambiguous Match for Gesetzesvorhaben: {:?}, using {:?} as query", result, self);
            Err(RetrievalError::AmbiguousMatch(format!("Gesetzesvorhaben: {:?}", result)).into())
        }else{
            Err(RetrievalError::NoMatch.into())
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Dokument {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ext_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub off_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub erstellt_am: Option<chrono::DateTime<Utc>>,
    pub letzter_zugriff: chrono::DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub pfad: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub gesetzesvorhaben: Option<Gesetzesvorhaben>,

    pub typ: String,
    pub file_type: String,
}

pub async fn find_doktyp_id(
    doktyp: &str,
    conn: &mut deadpool_diesel::postgres::Connection,
) -> Result<i32> {
    let query = dbcon::dokumenttypen::Update {
        name: Some(doktyp.to_owned()),
    };
    let result = dbcon::dokumenttypen::select_matching(conn, query).await.map_err(DatabaseError::from)?;
    if result.len() == 1 {
        Ok(result[0].id)
    } else if result.len() > 1 {
        tracing::warn!("Ambiguous Match for Doktypen: {:?}, using {:?} as query", result, doktyp);
        Err(RetrievalError::AmbiguousMatch(format!("Doktypen: {:?}", result)).into())
    }else{
        Err(RetrievalError::NoMatch.into())
    }
}
impl DatabaseInteraction for Dokument {
    async fn fetch_id(&self, conn: &mut deadpool_diesel::postgres::Connection) -> Result<i32> {
        let gesvh = if let Some(gesvh) = &self.gesetzesvorhaben{
            Some(gesvh.fetch_id(conn).await?)
        }else{None};
        let query = dbcon::dokumente::Update {
            ext_id: self.ext_id,
            accessed_at: Some(self.letzter_zugriff.naive_utc()),
            created_at: self.erstellt_am.map(|x| x.naive_utc()),
            path: Some(self.pfad.clone()),
            url: self.url.clone(),
            hash: self.hash.clone(),
            doktyp: Some(Some(find_doktyp_id(self.typ.as_str(), conn).await?)),
            filetype: Some(self.file_type.clone()),
            off_id: self.off_id.clone(),
            gesetzesvorhaben: Some(gesvh),
        };
        let result = dbcon::dokumente::select_matching(conn, query).await.map_err(DatabaseError::from)?;
        if result.len() == 1 {
            Ok(result[0].id)
        } else if result.len() > 1 {
            tracing::warn!("Ambiguous Match for Dokument: {:?}, using {:?} as query", result, self);
            Err(RetrievalError::AmbiguousMatch(format!("Dokument: {:?}", result)).into())
        }else{
            Err(RetrievalError::NoMatch.into())
        }
    }
}
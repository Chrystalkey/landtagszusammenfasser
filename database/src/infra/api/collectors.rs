use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::infra::db::connection as dbcon;
use crate::error::*;

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
    pub payload: Gesetzesvorhaben,
}

/// These are the response structures. A CUPResponse is sent to the collector to notify it of the state of data after the update.
#[derive(Serialize, Deserialize, Debug)]
pub struct CUPResponse {
    pub msg_id: Uuid,
    pub responding_to: Uuid,
    pub timestamp: chrono::DateTime<Utc>,
    pub payload: Gesetzesvorhaben,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Ausschuss {
    pub name: String,
    pub parlament: [char; 2],
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Status {
    pub name: String,
    pub parlament: [char; 2],
    pub datum: chrono::DateTime<Utc>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub dokumente: Vec<Dokument>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub schlagworte: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub eigenschaften: Vec<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub pfad: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub hash: Option<String>,

    pub letzter_zugriff: chrono::DateTime<Utc>,
    pub typ: String,
    pub file_type: String,
}

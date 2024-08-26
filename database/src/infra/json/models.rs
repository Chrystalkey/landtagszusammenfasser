use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

/// Sent from the collectors to the database
/// contains a collector-generated uuid, a timestamp and a list
/// of potentially new data.
/// If the collector does not know wether a specific piece of info is new or exists
/// the payload id is set to None (null) and the database checks entries for matches
pub struct CUPUpdate{
    msg_id: Uuid,
    timestamp: chrono::NaiveDateTime,
    payload: Vec<CUPPayload>,
}
pub struct CUPPayload{
    uuid: Option<Uuid>,
    data: CUPPayloadData
}
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
enum CUPPayloadData{
    Tops, 
    GesVH, 
    Dokument, 
    Ausschussberatung,
    Abstimmung
}

enum CUPRessourceState{
    Created, 
    Updated, 
    Exists, 
    CouldNotMatch, 
    Error(String)
}
pub struct CUPResponsePayload{
    uuid: Uuid,
    rs_state: CUPRessourceState
}

pub struct CUPResponse{
    msg_id: Uuid,
    responding_to: Uuid,
    timestamp: chrono::NaiveDateTime,
    payload: Vec<CUPResponsePayload>
}

mod updateable_entities{
    use uuid::Uuid;
    pub struct TOPs{
        ext_id: Uuid,
        datum: Option<chrono::NaiveDateTime>,
        url: Option<String>,
        parlament: Option<String>,
        tops: Option<Vec<Top>>
    }
    pub struct Parlament{
        kurz: [char;2]
    }
    pub struct Top{
        titel: String,
        dokument: Option<()>,
        abstimmung: Option<()>
    }
    pub struct Ausschuss{
        name: String,
        parlament: Parlament
    }
    pub struct Initiator{}
    pub struct Dokument{}
    pub struct Status{}
    pub struct GesEigenschaft{} 
    pub struct Gesetzesvorhaben{
        ext_id: Uuid,
        titel: String,
        off_titel: String, 
        url_gesblatt: Option<String>,
        id_gesblatt: Option<String>,
        verfassungsaendernd: Option<bool>,
        trojaner: Option<bool>,
        federfuehrung: Option<Ausschuss>,
        initiator: Option<Initiator>,
        dokumente: Vec<Dokument>,
        status: Vec<Status>,
        schlagworte: Vec<String>,
        eigenschaften: Vec<GesEigenschaft>,
        tops: Vec<Top>
    }
}
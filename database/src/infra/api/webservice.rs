use uuid::Uuid; 
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WSResponse{
    pub id: Uuid,
    pub payload: WSPayload
}

#[derive(Debug, Serialize)]
pub enum WSPayload{
    Gesetzesvorhaben(Vec<Gesetzesvorhaben>),
    Dokumente(Vec<Dokument>),
}

#[derive(Debug, Serialize)]
pub struct Dokument{
    pub id: Uuid,
    pub off_id: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub last_access: chrono::DateTime<chrono::Utc>,
    pub url: String,
    pub gesetzesvorhaben: Uuid,
    pub typ: String,
    pub dateityp: String,
}

#[derive(Debug, Serialize)]
pub struct Gesetzesvorhaben{
    pub id: Uuid,
    pub title: String,
    pub off_title: String,
    pub url_gesblatt: Option<String>,
    pub id_gesblatt: Option<String>,
    pub verfaender: bool,
    pub trojaner: bool,

    pub initiator: i32,
    pub federf_ausschuss: Option<i32>,

    pub status: Vec<i32>,
    pub schlagworte: Vec<i32>,
    pub eigenschaften: Vec<i32>,
    pub dokumente: Vec<Uuid>,
}
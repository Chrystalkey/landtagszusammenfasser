use openapi::models::*;
use openapi::apis::default::*;
use sqlx::PgTransaction;
use crate::{Result, LTZFServer};
use crate::db::*;

use super::objects;

pub async fn kal_get_by_date(
    date: chrono::NaiveDate,
    parlament: Parlament,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<KalDateGetResponse> {
    let dt_begin = date.and_time(chrono::NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap()).and_utc();
    let dt_end = date.checked_add_days(chrono::Days::new(1)).unwrap()
    .and_time(chrono::NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap()).and_utc();
    let sids = sqlx::query!("SELECT s.id FROM sitzung s 
    INNER JOIN gremium g ON g.id = s.gr_id
    INNER JOIN parlament p ON p.id = g.parl 
    WHERE termin BETWEEN $1 AND $2 AND p.value = $3",
    dt_begin, dt_end, parlament.to_string())
    .map(|r|r.id).fetch_all(&mut **tx).await?;
    if sids.is_empty() {
        return Ok(KalDateGetResponse::Status404_NotFound);
    }
    let mut vector = vec![];
    for sid in sids{
        vector.push(retrieve::sitzung_by_id(sid, tx).await?);
    }
    Ok(KalDateGetResponse::Status200_AntwortAufEineGefilterteAnfrageZuSitzungen(vector))
}

/// expects valid input, does no further date-input validation
pub async fn kal_put_by_date(
    date: chrono::NaiveDate,
    parlament: Parlament,
    sessions: Vec<Sitzung>,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<KalDatePutResponse> {
    let dt_begin = date.and_time(chrono::NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap()).and_utc();
    let dt_end = date.checked_add_days(chrono::Days::new(1)).unwrap()
    .and_time(chrono::NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap()).and_utc();
    // delete all entries that fit the description
    sqlx::query!("DELETE FROM sitzung WHERE sitzung.id = ANY(SELECT s.id FROM sitzung s 
    INNER JOIN gremium g ON g.id=s.gr_id 
    INNER JOIN parlament p ON p.id=g.parl 
    WHERE p.value = $1 AND s.termin BETWEEN $2 AND $3)",
    parlament.to_string(),dt_begin, dt_end).execute(&mut **tx).await?;

    // insert all entries
    for s in &sessions{
        insert::insert_sitzung(s, tx, srv).await?;
    }
    return Ok(KalDatePutResponse::Status201_Created);
}

pub async fn kal_get_by_param(
    qparams: KalGetQueryParams,
    hparams: KalGetHeaderParams,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<KalGetResponse> {todo!()}
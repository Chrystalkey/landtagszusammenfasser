use openapi::models::*;
use openapi::apis::default::*;
use sqlx::PgTransaction;
use crate::{Result, LTZFServer};
use crate::db::*;

pub async fn kal_get_by_date(
    date: chrono::NaiveDate,
    parlament: Parlament,
    tx: &mut PgTransaction<'_>,
    _srv: &LTZFServer,
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

pub fn find_applicable_date_range(
    y: Option<u32>,
    m: Option<u32>,
    d: Option<u32>,
    since: Option<chrono::DateTime<chrono::Utc>>,
    until: Option<chrono::DateTime<chrono::Utc>>,
    ifmodsince: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<(Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)> {
    let ymd_date_range = if let Some(y) = y{
        if let Some(m) = m{
            if let Some(d) = d{
                Some((chrono::NaiveDate::from_ymd_opt(y as i32,m,d).unwrap(), chrono::NaiveDate::from_ymd_opt(y as i32,m,d).unwrap()))
            }else{
                Some((chrono::NaiveDate::from_ymd_opt(y as i32,m,1).unwrap(), 
                    chrono::NaiveDate::from_ymd_opt(y as i32,m+1,1).unwrap().checked_sub_days(chrono::Days::new(1)).unwrap()))
            }
        }else{
            Some((
                chrono::NaiveDate::from_ymd_opt(y as i32, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(y as i32, 12, 31).unwrap()
            ))
        }
    }else{
        None
    }.map(|(a,b)| 
        (
            a.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            b.and_hms_opt(23, 59, 59).unwrap().and_utc(),
    ));

    let mut since_min = ifmodsince;
    let mut until_min = until;
    if since.is_some(){
        if since_min.is_some(){
            since_min = Some(since_min.unwrap().min(since.unwrap()));
        }else{
            since_min = since;
        }
    }
    if let Some((ymd_s, ymd_u)) = ymd_date_range{
        if since_min.is_some() { 
            since_min = Some(ymd_s.max(since_min.unwrap()));
        }else{
            since_min = Some(ymd_s);
        }
        if until_min.is_some(){
            until_min = Some(ymd_u.min(until_min.unwrap()));
        }else{
            until_min = Some(ymd_u);
        }
    }

    // semantic check
    if let Some(sm) = since_min{
        if sm < chrono::DateTime::parse_from_rfc3339("1945-01-01T00:00:00+00:00").unwrap(){
            return None;
        }
        if let Some(um) = until {
            if sm >= um {
                return None;
            }
        }
    }

    if let Some((ys, yu)) = ymd_date_range{
        if since_min.is_some() && since_min.unwrap() > yu {
            return None;
        }
        if until_min.is_some() && until_min.unwrap() < ys{
            return None;
        }
    }
    return Some((since_min, until_min));
}

pub async fn kal_get_by_param(
    qparams: KalGetQueryParams,
    hparams: KalGetHeaderParams,
    tx: &mut PgTransaction<'_>,
    _srv: &LTZFServer,
) -> Result<KalGetResponse> {
    // input validation
    let result = find_applicable_date_range(
        qparams.y.map(|x|x as u32),qparams.m.map(|x|x as u32),qparams.dom.map(|x|x as u32),
        qparams.since,qparams.until,hparams.if_modified_since);
    if result == None{
        return Ok(KalGetResponse::Status416_RequestRangeNotSatisfiable);
    }
    
    let params = retrieve::SitzungFilterParameters{
        gremium_like: qparams.gr,
        limit: qparams.limit.map(|x|x as u32),
        offset: qparams.offset.map(|x|x as u32),
        parlament: qparams.p,
        vgid: None,
        wp: qparams.wp.map(|x| x as u32),
        since: result.unwrap().0,
        until: result.unwrap().1
    };

    // retrieval
    let result = retrieve::sitzung_by_param(&params, tx).await?;
    if result.is_empty(){
        return Ok(KalGetResponse::Status204_NoContentFoundForTheSpecifiedParameters);
    }
    return Ok(KalGetResponse::Status200_AntwortAufEineGefilterteAnfrageZuSitzungen(result));
}

#[cfg(test)]
mod test{
    use super::find_applicable_date_range;
    use chrono::DateTime;
    
    #[test]
    fn test_date_range_none(){
        let result = find_applicable_date_range(None, None,None,None,None,None);
        assert!(result.is_some() && result.unwrap().0.is_none() && result.unwrap().1.is_none(), "None dates should not fail but produce (None, None)");
    }
    #[test]
    fn test_date_range_untilsince(){
        let since = DateTime::parse_from_rfc3339("1960-01-01T00:00:00+00:00").unwrap().to_utc();
        let until = DateTime::parse_from_rfc3339("1960-01-02T00:00:00+00:00").unwrap().to_utc();
        let result = find_applicable_date_range(None, None, None, 
            Some(since), Some(until),
            None);
        assert!(result.is_some() && result.unwrap().0 == Some(since) && result.unwrap().1 == Some(until), "Since and until should yield (since, until)")
    }
    #[test]
    fn test_date_range_ymd(){
        let y = 2012u32;
        let m = 5u32;
        let d = 12u32;

        // ymd
        let result = find_applicable_date_range(Some(y as u32), Some(m as u32), Some(d as u32),None, None, None);
        let expected_since = chrono::NaiveDate::from_ymd_opt(y as i32,m,d).unwrap()
        .and_hms_opt(0,0,0).unwrap().and_utc();
        let expected_until = chrono::NaiveDate::from_ymd_opt(y as i32,m,d).unwrap()
        .and_hms_opt(23,59,59).unwrap().and_utc();
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.0 == Some(expected_since) && result.1 == Some(expected_until), "ymd should start and end at the date range");
        // ym
        let result = find_applicable_date_range(Some(y as u32), Some(m as u32), None,None, None, None);
        let expected_since = chrono::NaiveDate::from_ymd_opt(y as i32,m,1).unwrap()
        .and_hms_opt(0,0,0).unwrap().and_utc();
        let expected_until = chrono::NaiveDate::from_ymd_opt(y as i32,m,31).unwrap()
        .and_hms_opt(23,59,59).unwrap().and_utc();
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.0 == Some(expected_since) && result.1 == Some(expected_until), "ymd should start and end at the date range");
        // y
        let result = find_applicable_date_range(Some(y as u32), None, None,None, None, None);
        let expected_since = chrono::NaiveDate::from_ymd_opt(y as i32,1,1).unwrap()
        .and_hms_opt(0,0,0).unwrap().and_utc();
        let expected_until = chrono::NaiveDate::from_ymd_opt(y as i32,12,31).unwrap()
        .and_hms_opt(23,59,59).unwrap().and_utc();
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.0 == Some(expected_since) && result.1 == Some(expected_until), "ymd should start and end at the date range");
    }

    #[test]
    fn test_minmax(){
        let y = 2012u32;
        
        let since = chrono::NaiveDate::from_ymd_opt(2000,3,1).unwrap()
        .and_hms_opt(0,0,0).unwrap().and_utc();
        let until = chrono::NaiveDate::from_ymd_opt(2012,7,31).unwrap()
        .and_hms_opt(15,59,59).unwrap().and_utc();

        let expected_since = chrono::NaiveDate::from_ymd_opt(2012,1,1).unwrap()
        .and_hms_opt(0,0,0).unwrap().and_utc();
        let expected_until = chrono::NaiveDate::from_ymd_opt(2012,7,31).unwrap()
        .and_hms_opt(15,59,59).unwrap().and_utc();
        
        let result = find_applicable_date_range(Some(y), None, None, Some(since), Some(until), None);
        assert!(result.is_some());
        let result =result.unwrap();
        assert!(result.0.is_some() && result.0.unwrap() == expected_since);
        assert!(result.1.is_some() && result.1.unwrap() == expected_until);
    }
}
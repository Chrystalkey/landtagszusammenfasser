use std::str::FromStr;

use crate::error::*;
use openapi::models;
use crate::utils::as_option;

pub async fn vorgang_by_id(id: i32, executor: &mut sqlx::PgTransaction<'_>) -> Result<models::Vorgang> {
    let pre_vg = sqlx::query!(
        "SELECT v.*, vt.value FROM vorgang v
        INNER JOIN vorgangstyp vt ON vt.id = v.typ
        WHERE v.id = $1", id)
    .fetch_one(&mut **executor).await?;

    let links = sqlx::query!("SELECT link FROM rel_vorgang_links WHERE vg_id = $1", id)
    .map(|row| row.link).fetch_all(&mut **executor).await?;

    let init_inst = sqlx::query!("SELECT initiator FROM rel_vorgang_init WHERE vg_id = $1", id)
    .map(|row| row.initiator).fetch_all(&mut **executor).await?;
    
    let init_prsn = sqlx::query!("SELECT initiator FROM rel_vorgang_init_person WHERE vg_id = $1", id)
    .map(|row| row.initiator).fetch_all(&mut **executor).await?;

    let ids = sqlx::query!("
    SELECT value as typ, identifikator as ident 
    FROM rel_vg_ident r
    INNER JOIN vg_ident_typ t ON t.id = r.typ
    WHERE r.vg_id = $1
    ORDER BY ident ASC", id)
        .map(|row| models::VgIdent{
        typ: models::VgIdentTyp::from_str(row.typ.as_str())
        .expect(format!("Could not convert database value `{}`into VgIdentTyp Variant", row.typ).as_str()),
        id: row.ident})
    .fetch_all(&mut **executor).await?;

    let station_ids = sqlx::query!("SELECT id FROM station WHERE vg_id = $1", id)
    .map(|row| row.id).fetch_all(&mut **executor).await?;

    let mut stationen = vec![];
    for sid in station_ids {
        stationen.push(station_by_id(sid, executor).await?);
    }

    Ok(models::Vorgang {
        api_id: pre_vg.api_id,
        titel: pre_vg.titel,
        kurztitel: pre_vg.kurztitel,
        wahlperiode: pre_vg.wahlperiode as u32,
        verfassungsaendernd: pre_vg.verfaend,
        typ: models::Vorgangstyp::from_str(pre_vg.value.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        initiatoren: init_inst,
        initiator_personen: as_option(init_prsn),
        ids: Some(ids),
        links: Some(links),
        stationen: stationen,
    })
}

pub async fn station_by_id(id: i32,  executor:&mut sqlx::PgTransaction<'_>) -> Result<models::Station> {
    let dokids = sqlx::query!("SELECT stat_id FROM rel_station_dokument WHERE dok_id = $1", id)
    .map(|r|r.stat_id).fetch_all(&mut **executor).await?;
    let mut doks = Vec::with_capacity(dokids.len());
    for did in dokids {
        doks.push(dokument_by_id(did, executor).await?.into());
    }
    let stlid = sqlx::query!("SELECT id FROM stellungnahme WHERE stat_id = $1", id)
    .map(|r|r.id).fetch_all(&mut **executor).await?;
    let mut stellungnahmen = Vec::with_capacity(stlid.len());
    for sid in stlid {
        stellungnahmen.push(stellungnahme_by_id(sid, executor).await?);
    }
    let sw = sqlx::query!(
        "SELECT DISTINCT(value) FROM rel_station_schlagwort r
        LEFT JOIN schlagwort sw ON sw.id = r.sw_id
        WHERE r.stat_id = $1
        ORDER BY value DESC", id)
    .map(|sw| sw.value).fetch_all(&mut **executor).await?;
    
    let bet_ges = sqlx::query!("SELECT gesetz FROM rel_station_gesetz WHERE stat_id = $1", id)
    .map(|r|r.gesetz).fetch_all(&mut **executor).await?;
    let temp_stat = sqlx::query!(
        "SELECT s.*, p.value as parlv, st.value as stattyp
        FROM station s
        INNER JOIN parlament p ON p.id = s.p_id
        INNER JOIN stationstyp st ON st.id = s.typ
        WHERE s.id=$1", id)
        .fetch_one(&mut **executor).await?;
    
    let gremium = sqlx::query!("
    SELECT p.value, g.name FROM gremium g INNER JOIN parlament p on p.id = g.parl
        WHERE g.id = $1", temp_stat.gr_id)
        .map(|x|models::Gremium{name: x.name, parlament: models::Parlament::from_str(&x.value).unwrap()})
        .fetch_optional(&mut **executor).await?;

    return Ok(models::Station {
        parlament: models::Parlament::from_str(temp_stat.parlv.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        typ: models::Stationstyp::from_str(temp_stat.stattyp.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        dokumente: doks,
        schlagworte: as_option(sw),
        stellungnahmen: as_option(stellungnahmen),
        start_zeitpunkt : temp_stat.start_zeitpunkt,
        letztes_update : Some(temp_stat.letztes_update),
        betroffene_texte: as_option(bet_ges),
        trojanergefahr: temp_stat.trojanergefahr.map(|x| x as u8),
        titel: temp_stat.titel,
        gremium,
        api_id: Some(temp_stat.api_id),
        link: temp_stat.link,
    });
}

pub async fn stellungnahme_by_id(
    id: i32,
    executor:&mut sqlx::PgTransaction<'_>,
) -> Result<models::Stellungnahme> {
    let temp = sqlx::query!("SELECT * FROM stellungnahme where id = $1", id).fetch_one(&mut **executor).await?;

    return Ok(models::Stellungnahme {
        dokument: dokument_by_id(temp.dok_id, executor).await?,
        meinung: temp.meinung as u8,
        lobbyregister_link: temp.lobbyreg_link,
    });
}

pub async fn dokument_by_id(id: i32,  executor:&mut sqlx::PgTransaction<'_>) -> Result<models::Dokument> {
    let rec = sqlx::query!(
        "SELECT d.*, value as typ_value FROM dokument d
        INNER JOIN dokumententyp dt ON dt.id = d.typ
        WHERE d.id = $1", id)
        .fetch_one(&mut **executor).await?;
    let schlagworte = sqlx::query!(
        "SELECT DISTINCT value 
        FROM rel_dok_schlagwort r
        LEFT JOIN schlagwort sw ON sw.id = r.sw_id
        WHERE dok_id = $1", id)
        .map(|r|r.value).fetch_all(&mut **executor).await?;
    let autoren = sqlx::query!("SELECT autor FROM rel_dok_autor WHERE dok_id = $1", id)
        .map(|r|r.autor).fetch_all(&mut **executor).await?;
    let autorpersonen = sqlx::query!("SELECT autor FROM rel_dok_autorperson WHERE dok_id = $1", id)
        .map(|r|r.autor).fetch_all(&mut **executor).await?;

    return Ok(models::Dokument {
        api_id: Some(rec.api_id),
        titel: rec.titel,
        kurztitel: rec.kurztitel,
        vorwort: rec.vorwort,
        volltext: rec.volltext,
        letzte_modifikation: rec.last_mod.into(),
        link: rec.link,
        hash: rec.hash,
        zusammenfassung: rec.zusammenfassung,
        schlagworte: as_option(schlagworte),
        autoren: as_option(autoren),
        autorpersonen: as_option(autorpersonen),
        typ: models::Doktyp::from_str(rec.typ_value.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        drucksnr: rec.drucksnr
    });
}

pub async fn top_by_id(id: i32, tx: &mut sqlx::PgTransaction<'_>) -> Result<models::Top>{
    todo!("top by id")
}
pub async fn ausschusssitzung_by_id(id: i32,  executor: &mut sqlx::PgTransaction<'_>) -> Result<models::Ausschusssitzung> {
    todo!("as by id")
}

pub async fn as_by_parameter(
    params: models::AsGetByIdHeaderParams,
    tx: &mut sqlx::PgTransaction<'_>) ->Result<Vec<models::Ausschusssitzung>>{
    todo!("AS_BY_PARAMETER")
}

pub async fn vorgang_by_parameter(
    params: models::VorgangGetQueryParams,
    hparam: models::VorgangGetHeaderParams,
    executor: &mut sqlx::PgTransaction<'_>
) -> Result<Vec<models::Vorgang>> {

    let vg_list = sqlx::query!(
        "WITH pre_table AS (
        SELECT vorgang.id, MAX(station.start_zeitpunkt) as lastmod FROM vorgang
            INNER JOIN vorgangstyp vt ON vt.id = vorgang.typ
            LEFT JOIN station ON station.vg_id = vorgang.id
            WHERE TRUE
            AND vorgang.wahlperiode = COALESCE($1, vorgang.wahlperiode)
            AND vt.value = COALESCE($2, vt.value)
        GROUP BY vorgang.id
        ORDER BY lastmod
        )
        SELECT * FROM pre_table WHERE
        lastmod > CAST(COALESCE($3, '1940-01-01T20:20:20Z') as TIMESTAMPTZ)
        OFFSET COALESCE($4, 0)
        LIMIT COALESCE($5, 64)",
    params.wp,
    params.vgtyp.map(|x|x.to_string()),
    hparam.if_modified_since.map(|x|x.to_rfc3339()),
    params.offset,
    params.limit)
    .map(|r|r.id)
    .fetch_all(&mut **executor).await?;

    let mut vector = Vec::with_capacity(vg_list.len());
    for id in vg_list{
        vector.push(super::retrieve::vorgang_by_id(id, executor).await?);
    }
    Ok(vector)
}



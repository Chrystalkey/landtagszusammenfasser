use std::str::FromStr;

use crate::error::*;
use openapi::models;
use crate::utils::as_option;

pub async fn vorgang_by_id(id: i32, executor: &mut sqlx::PgTransaction<'_>) -> Result<models::Vorgang> {
    let pre_vg = sqlx::query!(
        "SELECT vorgang.id as id, wahlperiode, api_id, verfaend as verfassungsaendernd, titel, vorgangstyp.api_key as typ FROM
        vorgang, vorgangstyp WHERE
        vorgang.typ = vorgangstyp.id AND
        vorgang.id = $1", id)
    .fetch_one(&mut **executor).await?;

    let links = sqlx::query!("SELECT link FROM rel_vorgang_links WHERE vorgang_id = $1", id)
    .map(|row| row.link).fetch_all(&mut **executor).await?;

    let init_inst = sqlx::query!("SELECT initiator FROM rel_vorgang_init WHERE vorgang_id = $1", id)
    .map(|row| row.initiator).fetch_all(&mut **executor).await?;
    
    let init_prsn = sqlx::query!("SELECT initiator FROM rel_vorgang_init_person WHERE vorgang_id = $1", id)
    .map(|row| row.initiator).fetch_all(&mut **executor).await?;

    let ids = sqlx::query!("SELECT api_key as typ, identifikator as ident FROM rel_vorgang_ident, vg_ident_typ
        WHERE rel_vorgang_ident.typ = vg_ident_typ.id AND rel_vorgang_ident.vorgang_id = $1", id)
        .map(|row| models::VgIdent{
        typ: models::VgIdentTyp::from_str(row.typ.as_str())
        .expect(format!("Could not convert database value `{}`into VgIdentTyp Variant", row.typ).as_str()),
        id: row.ident})
    .fetch_all(&mut **executor).await?;

    let station_ids = sqlx::query!("SELECT station.id as id FROM station, vorgang WHERE station.vorgang_id = vorgang.id AND vorgang.id = $1", id)
    .map(|row| row.id).fetch_all(&mut **executor).await?;

    let mut stationen = vec![];
    for sid in station_ids {
        stationen.push(station_by_id(sid, executor).await?);
    }

    Ok(models::Vorgang {
        api_id: pre_vg.api_id,
        titel: pre_vg.titel,
        wahlperiode: pre_vg.wahlperiode as u32,
        verfassungsaendernd: pre_vg.verfassungsaendernd,
        typ: models::Vorgangstyp::from_str(pre_vg.typ.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        initiatoren: init_inst,
        initiator_personen: as_option(init_prsn),
        ids: Some(ids),
        links: Some(links),
        stationen: stationen,
    })
}

pub async fn ausschusssitzung_by_id(id: i32,  executor: &mut sqlx::PgTransaction<'_>) -> Result<models::Ausschusssitzung> {
    let pre_as_rec = sqlx::query!("SELECT api_id, as_id, public, termin FROM ausschusssitzung WHERE id = $1", id)
    .fetch_one(&mut **executor).await?;

    let termin: crate::DateTime = pre_as_rec.termin.into();
    let ausschuss = sqlx::query!("SELECT name, api_key as parl FROM ausschuss, parlament WHERE parlament.id = ausschuss.parl_id AND ausschuss.id = $1", pre_as_rec.as_id)
    .map(|row|
        if let Ok(parl) = models::Parlament::from_str(&row.parl){
            Ok(models::Ausschuss{parlament: parl,name: row.name})
        }else{
            Err(DataValidationError::InvalidEnumValue { msg: format!("Tried to convert db val `{}` into parlament", row.parl) })
        }
    ).fetch_one(&mut **executor).await??;


    let pre_tops = sqlx::query!("SELECT top.id as id, top.titel, top.nummer as nummer FROM top
    JOIN rel_ass_tops ON rel_ass_tops.top_id = top.id AND rel_ass_tops.ass_id = $1;", id)
    .map(|row| (row.id, models::Top{
        titel: row.titel,
        nummer: row.nummer as u32,
        drucksachen: None,
        vorgang_id: None
    })).fetch_all(&mut **executor).await?;
    let mut tops = Vec::with_capacity(pre_tops.len());
    for (topid, top) in pre_tops {
        let doks_tops = sqlx::query!("SELECT dokument.id, dokument.drucksnr FROM dokument, tops_doks WHERE dokument.id = tops_doks.dok_id AND tops_doks.top_id = $1;", topid)
        .map(|row| (row.id, row.drucksnr)).fetch_all(&mut **executor).await?;
        
        let vg_nrs = sqlx::query!("SELECT vorgang.api_id, COUNT(rel_station_dokument.dok_id) as count FROM vorgang, station, rel_station_dokument
        WHERE vorgang.id = station.vorgang_id AND
        rel_station_dokument.stat_id = station.id AND
        rel_station_dokument.dok_id = ANY($1)
        GROUP BY vorgang.id ORDER BY count DESC;", &doks_tops.iter().map(|x|x.0).collect::<Vec<_>>())
        .map(|row| row.api_id)
        .fetch_one(&mut **executor).await?;
        let mut doks = Vec::with_capacity(doks_tops.len());
        for (id, drcks) in doks_tops {
            if let Some(_) = drcks {
                doks.push(dokument_by_id(id, executor).await?);
            }
        }
        let doks = doks.drain(..)
        .map(|d|models::TopDrucksachenInner::Dokument(Box::new(d)))
        .collect::<Vec<_>>();
        let top = models::Top{
            vorgang_id: Some(vg_nrs),
            drucksachen: as_option(doks),
            ..top
        };
        tops.push(top);
    }
    

    let experten = sqlx::query!("SELECT name, fachgebiet FROM experte, rel_ass_experten WHERE experte.id = rel_ass_experten.exp_id AND rel_ass_experten.ass_id = $1", id)
    .map(|row| models::Experte{
        name: row.name,
        fachgebiet: row.fachgebiet
    }).fetch_all(&mut **executor).await?;
    let experten = as_option(experten);

    Ok(models::Ausschusssitzung {
        api_id : Some(pre_as_rec.api_id),
        ausschuss,
        experten,
        public: pre_as_rec.public,
        termin,
        tops
    })
}

pub async fn station_by_id(id: i32,  executor:&mut sqlx::PgTransaction<'_>) -> Result<models::Station> {
    let dokids = sqlx::query!("SELECT stat_id FROM rel_station_dokument WHERE dok_id = $1", id)
    .map(|r|r.stat_id).fetch_all(&mut **executor).await?;
    let mut doks = Vec::with_capacity(dokids.len());
    for did in dokids {
        doks.push(dokument_by_id(did, executor).await?);
    }
    let stlid = sqlx::query!("SELECT id FROM stellungnahme WHERE stat_id = $1", id).map(|r|r.id).fetch_all(&mut **executor).await?;
    let mut stellungnahmen = Vec::with_capacity(stlid.len());
        for sid in stlid {
        stellungnahmen.push(stellungnahme_by_id(sid, executor).await?);
    }
    let sw = sqlx::query!(
        "SELECT DISTINCT(api_key) FROM rel_station_schlagwort 
        JOIN schlagwort ON schlagwort.id=rel_station_schlagwort.sw_id
        WHERE rel_station_schlagwort.stat_id = $1", id)
    .map(|sw| sw.api_key).fetch_all(&mut **executor).await?;
    
    let bet_ges = sqlx::query!("SELECT gesetz FROM rel_station_gesetz WHERE stat_id = $1", id)
    .map(|r|r.gesetz).fetch_all(&mut **executor).await?;
    let temp_stat = sqlx::query!(
    "SELECT parlament.api_key as parl, stationstyp.api_key as typ, zeitpunkt, trojanergefahr, link, titel, api_id, gremium FROM station
    JOIN parlament ON parlament.id = station.parl_id 
    JOIN stationstyp ON typ = stationstyp.id 
    WHERE station.id = $1", id).fetch_one(&mut **executor).await?;

    return Ok(models::Station {
        parlament: models::Parlament::from_str(temp_stat.parl.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        typ: models::Stationstyp::from_str(temp_stat.typ.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        dokumente: doks,
        schlagworte: as_option(sw),
        stellungnahmen: as_option(stellungnahmen),
        zeitpunkt : temp_stat.zeitpunkt,
        betroffene_texte: as_option(bet_ges),
        trojanergefahr: temp_stat.trojanergefahr.map(|x| x as u8),
        titel: temp_stat.titel,
        gremium: temp_stat.gremium,
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
        "SELECT api_id, titel, last_mod, link, hash, zusammenfassung, volltext, api_key as typ, drucksnr from dokument 
        JOIN dokumententyp ON dokumententyp.id = typ
        WHERE dokument.id = $1", id)
        .fetch_one(&mut **executor).await?;
    let schlagworte = sqlx::query!("SELECT DISTINCT api_key FROM rel_dok_schlagwort JOIN schlagwort ON schlagwort.id=rel_dok_schlagwort.sw_id WHERE dok_id = $1", id)
        .map(|r|r.api_key).fetch_all(&mut **executor).await?;
    let autoren = sqlx::query!("SELECT autor FROM rel_dok_autor WHERE dok_id = $1", id)
        .map(|r|r.autor).fetch_all(&mut **executor).await?;
    let autorpersonen = sqlx::query!("SELECT autor FROM rel_dok_autorperson WHERE dok_id = $1", id)
        .map(|r|r.autor).fetch_all(&mut **executor).await?;

    return Ok(models::Dokument {
        api_id: Some(rec.api_id),
        titel: rec.titel,
        letzte_modifikation: rec.last_mod.into(),
        link: rec.link,
        hash: rec.hash,
        zusammenfassung: rec.zusammenfassung,
        schlagworte: as_option(schlagworte),
        autoren: as_option(autoren),
        autorpersonen: as_option(autorpersonen),
        volltext: rec.volltext,
        typ: models::Doktyp::from_str(rec.typ.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        drucksnr: rec.drucksnr
    });
}

pub async fn vorgang_by_parameter(
    params: models::VorgangGetQueryParams,
    hparam: models::VorgangGetHeaderParams,
    executor: &mut sqlx::PgTransaction<'_>
) -> Result<Vec<models::Vorgang>> {

    let vg_list = sqlx::query!(
        "WITH pre_table AS (
        SELECT vorgang.id as id, MAX(station.zeitpunkt) as lastmod FROM vorgang
            JOIN vorgangstyp ON vorgang.typ = vorgangstyp.id
            JOIN station ON station.vorgang_id = vorgang.id
            WHERE TRUE
            AND vorgang.wahlperiode = COALESCE($1, vorgang.wahlperiode)
            AND vorgangstyp.api_key = COALESCE($2, vorgangstyp.api_key)
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

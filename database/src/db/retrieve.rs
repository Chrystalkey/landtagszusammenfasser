use std::str::FromStr;

use crate::error::*;
use crate::utils::as_option;
use openapi::models;

pub async fn vorgang_by_id(
    id: i32,
    executor: &mut sqlx::PgTransaction<'_>,
) -> Result<models::Vorgang> {
    let pre_vg = sqlx::query!(
        "SELECT v.*, vt.value FROM vorgang v
        INNER JOIN vorgangstyp vt ON vt.id = v.typ
        WHERE v.id = $1",
        id
    )
    .fetch_one(&mut **executor)
    .await?;

    let links = sqlx::query!(
        "SELECT link FROM rel_vorgang_links WHERE vg_id = $1 ORDER BY link ASC",
        id
    )
    .map(|row| row.link)
    .fetch_all(&mut **executor)
    .await?;

    let init_inst = sqlx::query!(
        "SELECT initiator FROM rel_vorgang_init WHERE vg_id = $1 ORDER BY initiator ASC",
        id
    )
    .map(|row| row.initiator)
    .fetch_all(&mut **executor)
    .await?;

    let init_prsn = sqlx::query!(
        "SELECT initiator FROM rel_vorgang_init_person WHERE vg_id = $1 ORDER BY initiator ASC",
        id
    )
    .map(|row| row.initiator)
    .fetch_all(&mut **executor)
    .await?;

    let ids = sqlx::query!(
        "
    SELECT value as typ, identifikator as ident 
    FROM rel_vg_ident r
    INNER JOIN vg_ident_typ t ON t.id = r.typ
    WHERE r.vg_id = $1
    ORDER BY ident ASC",
        id
    )
    .map(|row| models::VgIdent {
        typ: models::VgIdentTyp::from_str(row.typ.as_str()).expect(
            format!(
                "Could not convert database value `{}`into VgIdentTyp Variant",
                row.typ
            )
            .as_str(),
        ),
        id: row.ident,
    })
    .fetch_all(&mut **executor)
    .await?;

    let station_ids = sqlx::query!("SELECT id FROM station WHERE vg_id = $1", id)
        .map(|row| row.id)
        .fetch_all(&mut **executor)
        .await?;

    let mut stationen = vec![];
    for sid in station_ids {
        stationen.push(station_by_id(sid, executor).await?);
    }
    stationen.sort_by(|a, b| a.start_zeitpunkt.cmp(&b.start_zeitpunkt));

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

pub async fn station_by_id(
    id: i32,
    executor: &mut sqlx::PgTransaction<'_>,
) -> Result<models::Station> {
    let dokids = sqlx::query!(
        "SELECT dok_id FROM rel_station_dokument WHERE stat_id = $1",
        id
    )
    .map(|r| r.dok_id)
    .fetch_all(&mut **executor)
    .await?;
    let mut doks = Vec::with_capacity(dokids.len());
    for did in dokids {
        doks.push(dokument_by_id(did, executor).await?.into());
    }
    doks.sort_by(|a, b| match (a, b) {
        (models::DokRef::Dokument(a), models::DokRef::Dokument(b)) => a.link.cmp(&b.link),
        _ => {
            unreachable!("If this is the case document extraction failed")
        }
    });
    let stlid = sqlx::query!("SELECT id FROM stellungnahme WHERE stat_id = $1", id)
        .map(|r| r.id)
        .fetch_all(&mut **executor)
        .await?;
    let mut stellungnahmen = Vec::with_capacity(stlid.len());
    for sid in stlid {
        stellungnahmen.push(stellungnahme_by_id(sid, executor).await?);
    }
    stellungnahmen.sort_by(|a, b| a.dokument.link.cmp(&b.dokument.link));
    let sw = sqlx::query!(
        "SELECT DISTINCT(value) FROM rel_station_schlagwort r
        LEFT JOIN schlagwort sw ON sw.id = r.sw_id
        WHERE r.stat_id = $1
        ORDER BY value ASC",
        id
    )
    .map(|sw| sw.value)
    .fetch_all(&mut **executor)
    .await?;

    let bet_ges = sqlx::query!(
        "SELECT gesetz FROM rel_station_gesetz WHERE stat_id = $1",
        id
    )
    .map(|r| r.gesetz)
    .fetch_all(&mut **executor)
    .await?;
    let add_links = sqlx::query!(
        "SELECT link FROM rel_station_link WHERE stat_id = $1",
        id
    )
    .map(|r| r.link)
    .fetch_all(&mut **executor)
    .await?;
    let temp_stat = sqlx::query!(
        "SELECT s.*, p.value as parlv, st.value as stattyp
        FROM station s
        INNER JOIN parlament p ON p.id = s.p_id
        INNER JOIN stationstyp st ON st.id = s.typ
        WHERE s.id=$1",
        id
    )
    .fetch_one(&mut **executor)
    .await?;

    let gremium = sqlx::query!("
    SELECT p.value, g.name, g.wp, g.link, g.link_kalender FROM gremium g INNER JOIN parlament p on p.id = g.parl
        WHERE g.id = $1", temp_stat.gr_id)
        .map(|x|models::Gremium{
            name: x.name, wahlperiode: x.wp as u32,
            parlament: models::Parlament::from_str(&x.value).unwrap(),
            link: x.link, link_kalender: x.link_kalender
        })
        .fetch_optional(&mut **executor).await?;

    return Ok(models::Station {
        parlament: models::Parlament::from_str(temp_stat.parlv.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        typ: models::Stationstyp::from_str(temp_stat.stattyp.as_str())
            .map_err(|e| DataValidationError::InvalidEnumValue { msg: e })?,
        dokumente: doks,
        schlagworte: as_option(sw),
        stellungnahmen: as_option(stellungnahmen),
        start_zeitpunkt: temp_stat.start_zeitpunkt,
        letztes_update: Some(temp_stat.letztes_update),
        betroffene_texte: as_option(bet_ges),
        trojanergefahr: temp_stat.trojanergefahr.map(|x| x as u8),
        titel: temp_stat.titel,
        gremium,
        api_id: Some(temp_stat.api_id),
        link: temp_stat.link,
        additional_links: as_option(add_links),
        gremium_federf: temp_stat.gremium_isff
    });
}

pub async fn stellungnahme_by_id(
    id: i32,
    executor: &mut sqlx::PgTransaction<'_>,
) -> Result<models::Stellungnahme> {
    let temp = sqlx::query!("SELECT * FROM stellungnahme where id = $1", id)
        .fetch_one(&mut **executor)
        .await?;

    return Ok(models::Stellungnahme {
        dokument: dokument_by_id(temp.dok_id, executor).await?,
        meinung: temp.meinung as u8,
        lobbyregister_link: temp.lobbyreg_link,
    });
}

pub async fn dokument_by_id(
    id: i32,
    executor: &mut sqlx::PgTransaction<'_>,
) -> Result<models::Dokument> {
    tracing::debug!("Fetching dokument with id {}", id);
    let rec = sqlx::query!(
        "SELECT d.*, value as typ_value FROM dokument d
        INNER JOIN dokumententyp dt ON dt.id = d.typ
        WHERE d.id = $1",
        id
    )
    .fetch_one(&mut **executor)
    .await?;
    let schlagworte = sqlx::query!(
        "SELECT DISTINCT value 
        FROM rel_dok_schlagwort r
        LEFT JOIN schlagwort sw ON sw.id = r.sw_id
        WHERE dok_id = $1
        ORDER BY value ASC",
        id
    )
    .map(|r| r.value)
    .fetch_all(&mut **executor)
    .await?;
    let autoren = sqlx::query!(
        "SELECT autor FROM rel_dok_autor WHERE dok_id = $1 ORDER BY autor ASC",
        id
    )
    .map(|r| r.autor)
    .fetch_all(&mut **executor)
    .await?;
    let autorpersonen = sqlx::query!(
        "SELECT autor FROM rel_dok_autorperson WHERE dok_id = $1 ORDER BY autor ASC",
        id
    )
    .map(|r| r.autor)
    .fetch_all(&mut **executor)
    .await?;

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
        drucksnr: rec.drucksnr,
    });
}

/// the crucial part is how to find out which vg are connected to a DRCKS
/// if there exists a station which contains a document mentioned in the top, its vorgang is connected
pub async fn top_by_id(id: i32, tx: &mut sqlx::PgTransaction<'_>) -> Result<models::Top> {
    let scaffold = sqlx::query!("SELECT titel, nummer FROM top WHERE id = $1", id)
        .fetch_one(&mut **tx)
        .await?;
    // ds
    let dids = sqlx::query!("SELECT dok_id FROM tops_doks td WHERE top_id = $1", id)
        .map(|r| r.dok_id)
        .fetch_all(&mut **tx)
        .await?;
    let mut doks = vec![];
    for did in dids {
        doks.push(dokument_by_id(did, tx).await?.into());
    }
    doks.sort_by(|a, b| match (a, b) {
        (models::DokRef::Dokument(a), models::DokRef::Dokument(b)) => a.link.cmp(&b.link),
        _ => {
            unreachable!("If this is the case document extraction failed")
        }
    });
    // vgs
    let vgs = sqlx::query!(
        "
    SELECT DISTINCT(v.api_id) FROM station s    -- alle vorgänge von stationen,
INNER JOIN stationstyp st ON st.id = s.typ
INNER JOIN vorgang v ON v.id = s.vg_id
WHERE
EXISTS ( 									-- mit denen mindestens ein dokument assoziiert ist, dass hier auftaucht
	SELECT 1 FROM rel_station_dokument rsd 
	INNER JOIN tops_doks td ON td.dok_id = rsd.dok_id
	WHERE td.top_id = $1
)
    ORDER BY api_id ASC",
        id
    )
    .map(|r| r.api_id)
    .fetch_all(&mut **tx)
    .await?;

    return Ok(models::Top {
        nummer: scaffold.nummer as u32,
        titel: scaffold.titel,
        drucksachen: as_option(doks),
        vorgang_id: as_option(vgs),
    });
}

pub async fn ausschusssitzung_by_id(
    id: i32,
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<models::Ausschusssitzung> {
    let scaffold = sqlx::query!(
        "SELECT a.api_id, a.public, a.termin, p.value as plm, a.link as as_link, a.titel, a.nummer,
        g.name as grname, g.wp, g.link as gr_link, g.link_kalender FROM ausschusssitzung a
        INNER JOIN gremium g ON g.id = a.gr_id
        INNER JOIN parlament p ON p.id = g.parl 
        WHERE a.id = $1",
        id
    )
    .fetch_one(&mut **tx)
    .await?;
    // tops
    let topids = sqlx::query!("
    SELECT top.id FROM rel_ass_tops rat INNER JOIN top ON top.id = rat.top_id WHERE rat.ass_id = $1", id)
    .map(|r|r.id).fetch_all(&mut **tx).await?;
    let mut tops = vec![];
    for tid in topids {
        tops.push(top_by_id(tid, tx).await?);
    }
    tops.sort_by(|a, b| a.titel.cmp(&b.titel));
    // experten
    let experten = sqlx::query!(
        "SELECT e.name, e.fachgebiet FROM rel_ass_experten rae 
    INNER JOIN experte e ON rae.ass_id = $1 ORDER BY e.name ASC, e.fachgebiet ASC",
        id
    )
    .map(|r| models::Experte {
        fachgebiet: r.fachgebiet,
        name: r.name,
    })
    .fetch_all(&mut **tx)
    .await?;

    return Ok(models::Ausschusssitzung {
        api_id: Some(scaffold.api_id),
        public: scaffold.public,
        termin: scaffold.termin,
        ausschuss: models::Gremium {
            name: scaffold.grname,
            link: scaffold.gr_link,
            link_kalender: scaffold.link_kalender,
            wahlperiode: scaffold.wp as u32,
            parlament: models::Parlament::from_str(&scaffold.plm).unwrap(),
        },
        tops,
        link: scaffold.as_link,
        nummer: scaffold.nummer as u32,
        titel: scaffold.titel,
        experten: as_option(experten),
    });
}

pub async fn as_by_parameter(
    qparams: models::AsGetQueryParams,
    header_params: models::AsGetHeaderParams,
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<models::Ausschusssitzung>> {
    let lower_bnd = header_params.if_modified_since.map(|el| 
        if qparams.upd_since.is_some() {qparams.upd_since.unwrap().min(el)}else{el}
    );

    let as_list = sqlx::query!(
        "
    WITH pre_table AS (
        SELECT a.id, MAX(a.termin) as lastmod FROM  ausschusssitzung a
		INNER JOIN gremium g ON g.id = a.gr_id
		INNER JOIN parlament p ON p.id = g.parl
		WHERE p.value = COALESCE($1, p.value)
		AND g.wp = 		COALESCE($2, g.wp)
        GROUP BY a.id
        ORDER BY lastmod
        )

SELECT * FROM pre_table WHERE
lastmod > COALESCE($3, CAST('1940-01-01T20:20:20Z' as TIMESTAMPTZ)) AND
lastmod < COALESCE($4, NOW())
ORDER BY pre_table.lastmod ASC
OFFSET COALESCE($5, 0) 
LIMIT COALESCE($6, 64)
    ",
        qparams.parlament.map(|p| p.to_string()),
        qparams.wp,
        lower_bnd,
        qparams.upd_until,
        qparams.offset,
        qparams.limit
    )
    .map(|r| r.id)
    .fetch_all(&mut **tx)
    .await?;
    let mut vector = Vec::with_capacity(as_list.len());
    for id in as_list {
        vector.push(super::retrieve::ausschusssitzung_by_id(id, tx).await?);
    }
    return Ok(vector);
}

pub async fn vorgang_by_parameter(
    params: models::VorgangGetQueryParams,
    header_params: models::VorgangGetHeaderParams,
    executor: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<models::Vorgang>> {
    let lower_bnd = header_params.if_modified_since.map(|el| 
        if params.upd_since.is_some() {params.upd_since.unwrap().min(el)}else{el}
    );
    let vg_list = sqlx::query!(
        "WITH pre_table AS (
        SELECT vorgang.id, MAX(station.start_zeitpunkt) as lastmod FROM vorgang
            INNER JOIN vorgangstyp vt ON vt.id = vorgang.typ
            LEFT JOIN station ON station.vg_id = vorgang.id
			INNER JOIN parlament on parlament.id = station.p_id
            WHERE TRUE
            AND vorgang.wahlperiode = COALESCE($1, vorgang.wahlperiode)
            AND vt.value = COALESCE($2, vt.value)
			AND parlament.value= COALESCE($3, parlament.value)
			AND (CAST($4 as text) IS NULL OR EXISTS(SELECT 1 FROM rel_vorgang_init rvi WHERE rvi.initiator = $4))
			AND (CAST($5 as text) IS NULL OR EXISTS(SELECT 1 FROM rel_vorgang_init_person rvi WHERE rvi.initiator = $5))
        GROUP BY vorgang.id
        ORDER BY lastmod
        )
SELECT * FROM pre_table WHERE
lastmod > COALESCE($6, CAST('1940-01-01T20:20:20Z' as TIMESTAMPTZ)) 
AND lastmod < COALESCE($7, NOW())
ORDER BY pre_table.lastmod ASC
OFFSET COALESCE($8, 0) LIMIT COALESCE($9, 64)
",params.wp, params.vgtyp.map(|x|x.to_string()),
params.parlament.map(|p|p.to_string()),
params.init_contains, params.init_prsn_contains,
lower_bnd, params.upd_until, params.offset,
    params.limit)
    .map(|r|r.id)
    .fetch_all(&mut **executor).await?;

    let mut vector = Vec::with_capacity(vg_list.len());
    for id in vg_list {
        vector.push(super::retrieve::vorgang_by_id(id, executor).await?);
    }
    Ok(vector)
}

use std::str::FromStr;

use crate::{
    utils::{self, notify::notify_new_enum_entry},
    LTZFServer, Result,
};
use openapi::models;
use sqlx::PgTransaction;

/// Inserts a new Vorgang into the database.
pub async fn insert_vorgang(
    vg: &models::Vorgang,
    tx: &mut sqlx::PgTransaction<'_>,
    server: &LTZFServer,
) -> Result<i32> {
    tracing::info!("Inserting Complete Vorgang into the database");
    let obj = "vorgang";
    // master insert
    let vg_id = sqlx::query!(
        "
    INSERT INTO vorgang(api_id, titel, kurztitel, verfaend, wahlperiode, typ)
    VALUES
    ($1, $2, $3, $4, $5, (SELECT id FROM vorgangstyp WHERE value=$6))
    RETURNING vorgang.id;",
        vg.api_id,
        vg.titel,
        vg.kurztitel,
        vg.verfassungsaendernd,
        vg.wahlperiode as i32,
        server.guard_ts(vg.typ, vg.api_id, obj)?
    )
    .map(|r| r.id)
    .fetch_one(&mut **tx)
    .await?;

    // insert links
    sqlx::query!(
        "INSERT INTO rel_vorgang_links(link, vg_id) 
    SELECT val, $2 FROM UNNEST($1::text[]) as val",
        vg.links.as_ref().map(|x| &x[..]),
        vg_id
    )
    .execute(&mut **tx)
    .await?;

    // insert initiatoren
    let mut init_ids = vec![];
    for x in &vg.initiatoren{
        init_ids.push(insert_or_retrieve_autor(x, tx, server).await?);
    }
    sqlx::query!("INSERT INTO rel_vorgang_init(in_id, vg_id) SELECT val, $2 FROM UNNEST($1::int4[])as val;",
    &init_ids[..], vg_id)
    .execute(&mut **tx).await?;

    // insert ids
    let ident_list = vg
        .ids
        .as_ref()
        .map(|x| x.iter().map(|el| el.id.clone()).collect::<Vec<_>>());

    let identt_list = vg.ids.as_ref().map(|x| {
        x.iter()
            .map(|el| server.guard_ts(el.typ, vg.api_id, obj).unwrap())
            .collect::<Vec<_>>()
    });

    sqlx::query!(
        "INSERT INTO rel_vorgang_ident (vg_id, typ, identifikator) 
    SELECT $1, t.id, ident.ident FROM 
    UNNEST($2::text[], $3::text[]) as ident(ident, typ)
    INNER JOIN vg_ident_typ t ON t.value = ident.typ",
        vg_id,
        ident_list.as_ref().map(|x| &x[..]),
        identt_list.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx)
    .await?;

    // insert stations
    for stat in &vg.stationen {
        insert_station(stat.clone(), vg_id, tx, server).await?;
    }
    tracing::info!("Vorgang Insertion Successful with ID: {}", vg_id);
    Ok(vg_id)
}

pub async fn insert_station(
    stat: models::Station,
    vg_id: i32,
    tx: &mut sqlx::PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    // master insert
    let sapi = stat.api_id.unwrap_or(uuid::Uuid::now_v7());
    let obj = "station";
    if let Some(id) = sqlx::query!("SELECT id FROM station WHERE api_id = $1", sapi)
        .fetch_optional(&mut **tx)
        .await?
    {
        return Ok(id.id);
    }
    let gr_id = if let Some(gremium) = stat.gremium {
        let gr_id = insert_or_retrieve_gremium(&gremium, tx, srv).await?;
        Some(gr_id)
    } else {
        None
    };
    let stat_id = sqlx::query!(
        "INSERT INTO station 
        (api_id, gr_id, link, p_id, titel, trojanergefahr, typ, 
        zp_start, vg_id, zp_modifiziert, gremium_isff)
        VALUES
        ($1, $2, $3,
        (SELECT id FROM parlament   WHERE value = $4), $5, $6,
        (SELECT id FROM stationstyp WHERE value = $7), $8, $9, 
        COALESCE($10, NOW()), $11)
        RETURNING station.id",
        sapi, gr_id, stat.link,
        stat.parlament.to_string(), stat.titel, stat.trojanergefahr.map(|x|x as i32), srv.guard_ts(stat.typ, sapi, obj)?,
        stat.zp_start, vg_id, stat.zp_modifiziert, stat.gremium_federf
    ).map(|r|r.id)
    .fetch_one(&mut **tx).await?;

    // links
    sqlx::query!(
        "INSERT INTO rel_station_link(stat_id, link)
        SELECT $1, blub FROM UNNEST($2::text[]) as blub ON CONFLICT DO NOTHING",
        stat_id,
        stat.additional_links.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx)
    .await?;

    // assoziierte dokumente
    let mut did = Vec::with_capacity(stat.dokumente.len());
    for dokument in stat.dokumente {
        did.push(insert_or_retrieve_dok(&dokument, tx, srv).await?);
    }
    sqlx::query!(
        "INSERT INTO rel_station_dokument(stat_id, dok_id) 
    SELECT $1, blub FROM UNNEST($2::int4[]) as blub ON CONFLICT DO NOTHING",
        stat_id,
        &did[..]
    )
    .execute(&mut **tx)
    .await?;

    // stellungnahmen
    if let Some(stln) = stat.stellungnahmen {
        let mut doks = Vec::with_capacity(stln.len());
        for stln in stln {
            doks.push(insert_dokument(stln, tx, srv).await?);
        }
        sqlx::query!("INSERT INTO rel_station_stln (stat_id, dok_id)
        SELECT $1, did FROM UNNEST($2::int4[]) as did ON CONFLICT DO NOTHING",
        stat_id, &doks[..]
        ).execute(&mut **tx).await?;
    }
    // schlagworte
    insert_station_sw(stat_id, stat.schlagworte.unwrap_or(vec![]), tx).await?;

    return Ok(stat_id);
}

pub async fn insert_dokument(
    dok: models::Dokument,
    tx: &mut sqlx::PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    let dapi = dok.api_id.unwrap_or(uuid::Uuid::now_v7());
    match crate::db::merge::vorgang::dokument_merge_candidates(&dok, &mut **tx, srv).await? {
        super::merge::MergeState::OneMatch(id) => return Ok(id),
        super::merge::MergeState::AmbiguousMatch(matches) => {
            let api_ids = sqlx::query!(
                "SELECT api_id FROM dokument WHERE id = ANY($1::int4[])",
                &matches[..]
            )
            .map(|r| r.api_id)
            .fetch_all(&mut **tx)
            .await?;
            utils::notify::notify_ambiguous_match(api_ids, &dok, "insert_dokument", srv)?;
        }
        super::merge::MergeState::NoMatch => {}
    }
    let obj = "Dokument";
    let did = sqlx::query!(
        "INSERT INTO dokument(api_id, drucksnr, typ, titel, kurztitel, vorwort, 
        volltext, zusammenfassung, zp_lastmod, link, hash, zp_referenz, zp_created, meinung)
        VALUES(
            $1,$2, (SELECT id FROM dokumententyp WHERE value = $3),
            $4,$5,$6,$7,$8,$9,$10,$11, $12,$13,$14
        )RETURNING id",
        dapi, dok.drucksnr,  srv.guard_ts(dok.typ, dapi, obj)?, dok.titel, dok.kurztitel, dok.vorwort,
        dok.volltext,dok.zusammenfassung, dok.zp_modifiziert, dok.link, dok.hash,
        dok.zp_referenz, dok.zp_erstellt, dok.meinung.map(|r|r as i32)
    ).map(|r|r.id).fetch_one(&mut **tx).await?;

    // Schlagworte
    insert_dok_sw(did, dok.schlagworte.unwrap_or(vec![]), tx).await?;

    // authoren
    let mut aids = vec![];
    for a in &dok.autoren{
        aids.push(insert_or_retrieve_autor(a, tx, srv).await?)
    }
    sqlx::query!(
        "INSERT INTO rel_dok_autor(dok_id, aut_id) 
    SELECT $1, blub FROM UNNEST($2::int4[]) as blub ON CONFLICT DO NOTHING",
        did, &aids[..])
    .execute(&mut **tx)
    .await?;
    return Ok(did);
}

pub async fn insert_sitzung(
    ass: &models::Sitzung,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    let api_id = ass.api_id.unwrap_or(uuid::Uuid::now_v7());

    // gremium insert or fetch
    let gr_id = insert_or_retrieve_gremium(&ass.gremium, tx, srv).await?;
    // master insert
    let id = sqlx::query!(
        "INSERT INTO sitzung 
        (api_id, termin, public, gr_id, link, nummer, titel)
    VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        api_id,
        ass.termin,
        ass.public,
        gr_id,
        ass.link,
        ass.nummer as i32,
        ass.titel
    )
    .map(|r| r.id)
    .fetch_one(&mut **tx)
    .await?;
    // insert tops
    for top in &ass.tops {
        insert_top(id, &top, tx, srv).await?;
    }

    // insert experten
    let mut exp_ids = vec![];
    for exp in ass.experten.as_ref().unwrap_or(&vec![]) {
        let ex_id = insert_or_retrieve_autor(exp, tx, srv).await?;
        exp_ids.push(ex_id);
    }
    sqlx::query!(
        "INSERT INTO rel_sitzung_experten(sid, eid)
    SELECT $1, eids FROM UNNEST($2::int4[]) as eids",
        id,
        &exp_ids[..]
    )
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

pub async fn insert_top(
    sid: i32,
    top: &models::Top,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    // master insert
    let tid = sqlx::query!(
        "INSERT INTO top(titel, nummer, sid) VALUES($1, $2, $3) RETURNING id;",
        top.titel,
        top.nummer as i32,
        sid
    )
    .map(|r| r.id)
    .fetch_one(&mut **tx)
    .await?;

    // drucksachen
    let mut dids = vec![];
    for d in top.dokumente.as_ref().unwrap_or(&vec![]) {
        dids.push(insert_or_retrieve_dok(&d, tx, srv).await?);
    }
    sqlx::query!(
        "INSERT INTO tops_doks(top_id, dok_id)
    SELECT $1, did FROM UNNEST($2::int4[]) as did",
        tid,
        &dids[..]
    )
    .execute(&mut **tx)
    .await?;

    return Ok(tid);
}

pub async fn insert_or_retrieve_gremium(
    gr: &models::Gremium,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    let gid = sqlx::query!(
        "SELECT g.id FROM gremium g, parlament p WHERE
    g.name = $1 AND 
    p.id = g.parl AND  p.value = $2
    AND g.wp = $3",
        gr.name,
        gr.parlament.to_string(),
        gr.wahlperiode as i32
    )
    .map(|r| r.id)
    .fetch_optional(&mut **tx)
    .await?;
    if gid.is_some() {
        return Ok(gid.unwrap());
    }

    let similarity = sqlx::query!(
        "SELECT g.wp,g.name, SIMILARITY(name, $1) as sim, g.link
    FROM gremium g, parlament p
    WHERE SIMILARITY(name, $1) > 0.66 AND 
    g.parl = p.id AND p.value = $2",
        gr.name,
        gr.parlament.to_string()
    )
    .map(|r| {
        (
            r.sim.unwrap(),
            models::Gremium {
                link: r.link,
                parlament: gr.parlament,
                wahlperiode: r.wp as u32,
                name: r.name,
            },
        )
    })
    .fetch_all(&mut **tx)
    .await?;
    notify_new_enum_entry(gr, similarity, srv)?;
    let id = sqlx::query!(
        "INSERT INTO gremium(name, parl, wp, link) VALUES 
    ($1, (SELECT id FROM parlament p WHERE p.value = $2), $3, $4) 
    RETURNING gremium.id",
        gr.name,
        gr.parlament.to_string(),
        gr.wahlperiode as i32,
        gr.link
    )
    .map(|r| r.id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

pub async fn insert_or_retrieve_autor(
    at: &models::Autor,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    let eid = sqlx::query!(
        "SELECT a.id FROM autor a WHERE 
        ((a.person IS NULL AND $1::text IS NULL) OR a.person = $1) AND 
        ((a.organisation IS NULL AND $2::text IS NULL) OR a.organisation = $2) AND 
        ((a.fachgebiet IS NULL AND $3::text IS NULL) OR a.fachgebiet = $3)",
        at.person,
        at.organisation,
        at.fachgebiet
    )
    .map(|r| r.id)
    .fetch_optional(&mut **tx)
    .await?;
    if eid.is_some() {
        return Ok(eid.unwrap());
    }

    let similarity = sqlx::query!(
        "
        WITH similarities AS (
            SELECT id, 
            SIMILARITY(person, $1) as p, 
            SIMILARITY(organisation, $2) as o, 
            SIMILARITY(fachgebiet, $3) as f
            FROM autor a
        )
        SELECT a.*, 
        CASE WHEN s.p IS NOT NULL THEN s.p
        ELSE s.o END AS sim
        
        FROM autor a 
        INNER JOIN similarities s ON s.id = a.id
        
        WHERE 
        
        (($1 IS NULL AND a.person IS NULL) OR s.p > 0.66) AND 
        s.o > 0.66 AND
        (($3 IS NULL AND a.fachgebiet IS NULL) OR s.f > 0.66)",
        at.person,
        at.organisation,
        at.fachgebiet
    )
    .map(|r| {
        (
            r.sim.unwrap(),
            models::Autor {
                fachgebiet: r.fachgebiet,
                person: r.person,
                organisation: r.organisation,
                lobbyregister: r.lobbyregister,
            },
        )
    })
    .fetch_all(&mut **tx)
    .await?;
    notify_new_enum_entry(at, similarity, srv)?;
    let id = sqlx::query!(
        "INSERT INTO autor(person, organisation, lobbyregister, fachgebiet) 
        VALUES ($1, $2, $3, $4) RETURNING autor.id",
        at.person,
        at.organisation,
        at.lobbyregister,
        at.fachgebiet,
    )
    .map(|r| r.id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

pub async fn insert_or_retrieve_dok(
    dr: &models::DokRef,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    match dr {
        models::DokRef::Dokument(dok) => Ok(insert_dokument((**dok).clone(), tx, srv).await?),
        models::DokRef::String(dapi_id) => {
            let api_id = uuid::Uuid::from_str(dapi_id.as_str())?;
            Ok(
                sqlx::query!("SELECT id FROM dokument WHERE api_id = $1", api_id)
                    .map(|r| r.id)
                    .fetch_one(&mut **tx)
                    .await?,
            )
        }
    }
}
pub async fn insert_station_sw(
    sid: i32,
    sw: Vec<String>,
    tx: &mut PgTransaction<'_>,
) -> Result<()> {
    let sw: Vec<_> = sw.iter().map(|s| s.trim().to_lowercase()).collect();
    sqlx::query!(
        "
    WITH 
    existing_ids AS (SELECT DISTINCT id FROM schlagwort WHERE value = ANY($1::text[])),
    inserted AS (
        INSERT INTO schlagwort(value) 
        SELECT DISTINCT(key) FROM UNNEST($1::text[]) as key
        ON CONFLICT DO NOTHING
        RETURNING id
    ),
    allofthem AS(
        SELECT id FROM inserted UNION SELECT id FROM existing_ids
    )

    INSERT INTO rel_station_schlagwort(stat_id, sw_id)
    SELECT $2, allofthem.id FROM allofthem
    ON CONFLICT DO NOTHING",
        &sw[..],
        sid
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
pub async fn insert_dok_sw(did: i32, sw: Vec<String>, tx: &mut PgTransaction<'_>) -> Result<()> {
    let sw: Vec<_> = sw.iter().map(|s| s.trim().to_lowercase()).collect();
    sqlx::query!(
        "
    WITH 
    existing_ids AS (SELECT DISTINCT id FROM schlagwort WHERE value = ANY($1::text[])),
    inserted AS (
        INSERT INTO schlagwort(value) 
        SELECT DISTINCT(key) FROM UNNEST($1::text[]) as key
        ON CONFLICT DO NOTHING
        RETURNING id
    ),
    allofthem AS(
        SELECT id FROM inserted UNION SELECT id FROM existing_ids
    )

    INSERT INTO rel_dok_schlagwort(dok_id, sw_id)
    SELECT $2, allofthem.id FROM allofthem
    ON CONFLICT DO NOTHING",
        &sw[..],
        did
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

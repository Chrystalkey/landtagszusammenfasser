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
    tracing::info!("Inserting complete Vorgang into the database");
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
    sqlx::query!("INSERT INTO rel_vorgang_init(initiator, vg_id) SELECT val, $2 FROM UNNEST($1::text[])as val;",
    &vg.initiatoren[..], vg_id)
    .execute(&mut **tx).await?;
    sqlx::query!("INSERT INTO rel_vorgang_init_person(initiator, vg_id) SELECT val, $2 FROM UNNEST($1::text[])as val;",
    vg.initiator_personen.as_ref().map(|x|&x[..]), vg_id)
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
        "INSERT INTO rel_vg_ident (vg_id, typ, identifikator) 
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
    tracing::info!("Insertion Successful with ID: {}", vg_id);
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
        (api_id, gr_id, link, p_id, titel, trojanergefahr, typ, start_zeitpunkt, vg_id, letztes_update)
        VALUES
        ($1, $2, $3,
        (SELECT id FROM parlament   WHERE value = $4), $5, $6,
        (SELECT id FROM stationstyp WHERE value = $7), $8, $9, COALESCE($10, NOW()))
        RETURNING station.id",
        sapi, gr_id, stat.link,
        stat.parlament.to_string(), stat.titel, stat.trojanergefahr.map(|x|x as i32), srv.guard_ts(stat.typ, sapi, obj)?,
        stat.start_zeitpunkt, vg_id, stat.letztes_update
    ).map(|r|r.id)
    .fetch_one(&mut **tx).await?;

    // betroffene gesetzestexte
    sqlx::query!(
        "INSERT INTO rel_station_gesetz(stat_id, gesetz)
        SELECT $1, blub FROM UNNEST($2::text[]) as blub ON CONFLICT DO NOTHING",
        stat_id,
        stat.betroffene_texte.as_ref().map(|x| &x[..])
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
        let mut mng = Vec::with_capacity(stln.len());
        let mut doks = Vec::with_capacity(stln.len());
        let mut lobby = Vec::with_capacity(stln.len());
        for stln in stln {
            mng.push(stln.meinung as i32);
            lobby.push(stln.lobbyregister_link);
            doks.push(insert_dokument(stln.dokument, tx, srv).await?);
        }
        sqlx::query!("INSERT INTO stellungnahme (stat_id, meinung, lobbyreg_link, dok_id)
        SELECT $1, mn, lobby, did FROM UNNEST($2::int4[], $3::text[], $4::int4[]) as blub(mn, lobby, did)",
        stat_id, &mng[..], &lobby as &[Option<String>], &doks[..]
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
        "INSERT INTO dokument(api_id, drucksnr, typ, titel, kurztitel, vorwort, volltext, zusammenfassung, last_mod, link, hash)
        VALUES(
            $1,$2, (SELECT id FROM dokumententyp WHERE value = $3),
            $4,$5,$6,$7,$8,$9,$10,$11
        )RETURNING id",
        dapi, dok.drucksnr,  srv.guard_ts(dok.typ, dapi, obj)?, dok.titel, dok.kurztitel, dok.vorwort,
        dok.volltext,dok.zusammenfassung, dok.letzte_modifikation, dok.link, dok.hash
    ).map(|r|r.id).fetch_one(&mut **tx).await?;

    // Schlagworte
    insert_dok_sw(did, dok.schlagworte.unwrap_or(vec![]), tx).await?;

    // authoren
    sqlx::query!(
        "INSERT INTO rel_dok_autor(dok_id, autor) 
    SELECT $1, blub FROM UNNEST($2::text[]) as blub",
        did,
        dok.autoren.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        "INSERT INTO rel_dok_autorperson(dok_id, autor) 
    SELECT $1, blub FROM UNNEST($2::text[]) as blub",
        did,
        dok.autorpersonen.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx)
    .await?;
    return Ok(did);
}

pub async fn insert_ausschusssitzung(
    ass: &models::Ausschusssitzung,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    let api_id = ass.api_id.unwrap_or(uuid::Uuid::now_v7());

    // gremium insert or fetch
    let gr_id = insert_or_retrieve_gremium(&ass.ausschuss, tx, srv).await?;
    // master insert
    let id = sqlx::query!(
        "INSERT INTO ausschusssitzung (api_id, termin, public, gr_id, link, nummer, titel)
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
    let mut tids = vec![];
    for top in &ass.tops {
        tids.push(insert_top(&top, tx, srv).await?);
    }
    sqlx::query!(
        "INSERT INTO rel_ass_tops(ass_id, top_id) 
    SELECT $1, tids FROM UNNEST($2::int4[]) as tids",
        id,
        &tids[..]
    )
    .execute(&mut **tx)
    .await?;

    // insert experten
    let mut exp_ids = vec![];
    for exp in ass.experten.as_ref().unwrap_or(&vec![]) {
        let ex_id = insert_or_retrieve_experte(exp, tx, srv).await?;
        exp_ids.push(ex_id);
    }
    sqlx::query!(
        "INSERT INTO rel_ass_experten(ass_id, exp_id)
    SELECT $1, eids FROM UNNEST($2::int4[]) as eids",
        id,
        &exp_ids[..]
    )
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

pub async fn insert_top(
    top: &models::Top,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    // master insert
    let tid = sqlx::query!(
        "INSERT INTO top(titel, nummer) VALUES($1, $2)RETURNING id;",
        top.titel,
        top.nummer as i32
    )
    .map(|r| r.id)
    .fetch_one(&mut **tx)
    .await?;

    // drucksachen
    let mut dids = vec![];
    for d in top.drucksachen.as_ref().unwrap_or(&vec![]) {
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
        "SELECT g.wp,g.name, SIMILARITY(name, $1) as sim, g.link, g.link_kalender
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
                link_kalender: r.link_kalender,
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
        "INSERT INTO gremium(name, parl, wp, link, link_kalender) VALUES 
    ($1, (SELECT id FROM parlament p WHERE p.value = $2), $3, $4, $5) 
    RETURNING gremium.id",
        gr.name,
        gr.parlament.to_string(),
        gr.wahlperiode as i32,
        gr.link,
        gr.link_kalender
    )
    .map(|r| r.id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

pub async fn insert_or_retrieve_experte(
    ex: &models::Experte,
    tx: &mut PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<i32> {
    let eid = sqlx::query!(
        "SELECT e.id FROM experte e WHERE e.name = $1 AND e.fachgebiet = $2",
        ex.name,
        ex.fachgebiet
    )
    .map(|r| r.id)
    .fetch_optional(&mut **tx)
    .await?;
    if eid.is_some() {
        return Ok(eid.unwrap());
    }

    let similarity = sqlx::query!(
        "SELECT *, SIMILARITY(name, $1) as sim FROM experte e 
    WHERE SIMILARITY(name, $1) > 0.66 AND SIMILARITY(fachgebiet, $2) > 0.66",
        ex.name,
        ex.fachgebiet
    )
    .map(|r| {
        (
            r.sim.unwrap(),
            models::Experte {
                fachgebiet: r.fachgebiet,
                name: r.name,
            },
        )
    })
    .fetch_all(&mut **tx)
    .await?;
    notify_new_enum_entry(ex, similarity, srv)?;
    let id = sqlx::query!(
        "INSERT INTO experte(name, fachgebiet) VALUES ($1, $2) RETURNING experte.id",
        ex.name,
        ex.fachgebiet
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

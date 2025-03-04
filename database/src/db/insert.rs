use openapi::models;
use uuid::Uuid;
use crate::{LTZFServer, Result};

/// Inserts a new GSVH into the database.
pub async fn insert_vorgang(
    vg: &models::Vorgang,
    tx: &mut sqlx::PgTransaction<'_>,
    server: &LTZFServer,
) -> Result<i32> {
    tracing::info!("Inserting complete Vorgang into the database");
    let obj = "vorgang";
    // master insert
    let vg_id = sqlx::query!("
    INSERT INTO vorgang(api_id, titel, kurztitel, verfaend, wahlperiode, typ)
    VALUES
    ($1, $2, $3, $4, $5, (SELECT id FROM vorgangstyp WHERE value=$6))
    RETURNING vorgang.id;", 
    vg.api_id, vg.titel, vg.kurztitel, vg.verfassungsaendernd, vg.wahlperiode as i32, 
    server.guard_ts(vg.typ, vg.api_id,obj)?)
    .map(|r|r.id).fetch_one(&mut **tx).await?;

    // insert links
    sqlx::query!("INSERT INTO rel_vorgang_links(link, vg_id) 
    SELECT val, $2 FROM UNNEST($1::text[]) as val", 
    vg.links.as_ref().map(|x| &x[..]), vg_id)
    .execute(&mut **tx).await?;

    // insert initiatoren
    sqlx::query!("INSERT INTO rel_vorgang_init(initiator, vg_id) SELECT val, $2 FROM UNNEST($1::text[])as val;", 
    &vg.initiatoren[..], vg_id)
    .execute(&mut **tx).await?;
    sqlx::query!("INSERT INTO rel_vorgang_init_person(initiator, vg_id) SELECT val, $2 FROM UNNEST($1::text[])as val;", 
    vg.initiator_personen.as_ref().map(|x|&x[..]), vg_id)
    .execute(&mut **tx).await?;

    // insert ids
    let ident_list = vg.ids.as_ref().map(|x|x.iter()
    .map(|el|el.id.clone()).collect::<Vec<_>>());

    let identt_list = vg.ids.as_ref().map(|x|x.iter()
    .map(|el| server.guard_ts(el.typ, vg.api_id, obj).unwrap()).collect::<Vec<_>>());

    sqlx::query!("INSERT INTO rel_vg_ident (vg_id, typ, identifikator) 
    SELECT $1, t.id, ident.ident FROM 
    UNNEST($2::text[], $3::text[]) as ident(ident, typ)
    INNER JOIN vg_ident_typ t ON t.value = ident.typ",
    vg_id, ident_list.as_ref().map(|x| &x[..]), identt_list.as_ref().map(|x| &x[..]))
    .execute(&mut **tx).await?;
    
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
    srv: &LTZFServer
) -> Result<i32> {
    // master insert
    let sapi = stat.api_id.unwrap_or(uuid::Uuid::now_v7());
    let obj= "station";
    if let Some(id) = sqlx::query!("SELECT id FROM station WHERE api_id = $1", sapi).fetch_optional(&mut **tx).await?{
        return Ok(id.id);
    }
    let gr_id = if let Some(gremium) = stat.gremium{
        let id = sqlx::query!("INSERT INTO gremium(name, parl) VALUES ($1, (SELECT id FROM parlament WHERE value=$2)) ON CONFLICT DO NOTHING RETURNING id",
        gremium.name, gremium.parlament.to_string()).map(|r|r.id).fetch_one(&mut **tx).await?;
        Some(id)
    }else {
        None
    };
    let stat_id = sqlx::query!(
        "INSERT INTO station 
        (api_id, gr_id, link, p_id, titel, trojanergefahr, typ, start_zeitpunkt, vg_id, letztes_update)
        VALUES
        ($1, $2, $3, 
        (SELECT id FROM parlament   WHERE value = $4), $5, $6, 
        (SELECT id FROM stationstyp WHERE value = $7), $8, $9, $10)
        RETURNING station.id", 
        sapi, gr_id, stat.link,
        stat.parlament.to_string(), stat.titel, stat.trojanergefahr.map(|x|x as i32), srv.guard_ts(stat.typ, sapi, obj)?,
        stat.start_zeitpunkt, vg_id, stat.letztes_update
    ).map(|r|r.id)
    .fetch_one(&mut **tx).await?;

    // betroffene gesetzestexte
    sqlx::query!(
        "INSERT INTO rel_station_gesetz(stat_id, gesetz)
        SELECT $1, blub FROM UNNEST($2::text[]) as blub",
        stat_id, stat.betroffene_texte.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx).await?;

    // assoziierte dokumente
    let mut did = Vec::with_capacity(stat.dokumente.len());
    for dokument in stat.dokumente{
        did.push(
            match dokument{
                models::DokRef::String(s) => {
                    let uuid = Uuid::parse_str(&*s)?;
                    if let Some(id) = sqlx::query!("SELECT id FROM dokument WHERE api_id = $1", uuid)
                    .map(|r|r.id).fetch_optional(&mut **tx).await?{
                        id
                    }else{
                        return Err(crate::error::LTZFError::Validation { source: crate::error::DataValidationError::IncompleteDataSupplied { input: *s } })
                    }
                },
                models::DokRef::Dokument(d) => {insert_dokument(*d, tx, srv).await?}
            }
        );
    }
    sqlx::query!("INSERT INTO rel_station_dokument(stat_id, dok_id) 
    SELECT $1, blub FROM UNNEST($2::int4[]) as blub", stat_id, &did[..])
    .execute(&mut **tx).await?;

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
    sqlx::query!("
        WITH 
        existing_ids AS (SELECT DISTINCT id FROM schlagwort WHERE value = ANY($1::text[])),
        inserted AS(
            INSERT INTO schlagwort(value) 
            SELECT DISTINCT(key) FROM UNNEST($1::text[]) as key
            ON CONFLICT DO NOTHING
            RETURNING id
        ),
        allofthem AS(
            SELECT id FROM inserted UNION SELECT id FROM existing_ids
        )

        INSERT INTO rel_station_schlagwort(stat_id, sw_id)
        SELECT $2, allofthem.id FROM allofthem",
        stat.schlagworte.as_ref().map(|x|&x[..]), stat_id
    )
    .execute(&mut **tx).await?;

    return Ok(stat_id);
}

pub async fn insert_dokument(
    dok: models::Dokument,
    tx: &mut sqlx::PgTransaction<'_>,
    srv: &LTZFServer) 
    -> Result<i32> {
    let dapi = dok.api_id.unwrap_or(uuid::Uuid::now_v7());
    if let Some(id) = sqlx::query!("SELECT id FROM dokument WHERE api_id = $1", dapi).fetch_optional(&mut **tx).await?{
        return Ok(id.id);
    } 
    let obj= "Dokument";
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
    sqlx::query!("
        WITH existing_ids AS (SELECT DISTINCT id FROM schlagwort WHERE value = ANY($1::text[])),
        inserted AS(
            INSERT INTO schlagwort(value) 
            SELECT DISTINCT(key) FROM UNNEST($1::text[]) as key
            ON CONFLICT DO NOTHING
            RETURNING id
        ),
        allofthem AS(
            SELECT id FROM inserted UNION SELECT id FROM existing_ids
        )

        INSERT INTO rel_dok_schlagwort(dok_id, sw_id)
        SELECT $2, allofthem.id FROM allofthem",
        dok.schlagworte.as_ref().map(|x|&x[..]), did
    )
    .execute(&mut **tx).await?;

    // authoren
    sqlx::query!("INSERT INTO rel_dok_autor(dok_id, autor) 
    SELECT $1, blub FROM UNNEST($2::text[]) as blub", did, 
    dok.autoren.as_ref().map(|x|&x[..]))
    .execute(&mut **tx).await?;
    sqlx::query!("INSERT INTO rel_dok_autorperson(dok_id, autor) 
    SELECT $1, blub FROM UNNEST($2::text[]) as blub", 
    did, dok.autorpersonen.as_ref().map(|x|&x[..]))
    .execute(&mut **tx).await?;
    return Ok(did);
}
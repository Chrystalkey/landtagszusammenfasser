use openapi::models;
use crate::Result;

/// Inserts a new GSVH into the database.
pub async fn insert_vorgang(
    vg: &models::Vorgang,
    tx: &mut sqlx::PgTransaction<'_>
) -> Result<i32> {
    tracing::info!("Inserting complete Vorgang into the database");
    // master insert
    let vg_id = sqlx::query!("
    INSERT INTO vorgang(api_id, titel, verfaend, wahlperiode, typ)
    VALUES
    ($1, $2, $3, $4, (SELECT id FROM vorgangstyp WHERE api_key=$5))
    RETURNING vorgang.id;", 
    vg.api_id, vg.titel, vg.verfassungsaendernd, vg.wahlperiode as i32, vg.typ.to_string()
    ).map(|r|r.id).fetch_one(&mut **tx).await?;

    // insert links
    sqlx::query!("INSERT INTO rel_vorgang_links(link, vorgang_id) SELECT val, $2 FROM UNNEST($1::text[]) as val", 
    vg.links.as_ref().map(|x| &x[..]), vg_id)
    .execute(&mut **tx).await?;

    // insert initiatoren
    sqlx::query!("INSERT INTO rel_vorgang_init(initiator, vorgang_id) SELECT val, $2 FROM UNNEST($1::text[])as val;", 
    &vg.initiatoren[..], vg_id)
    .execute(&mut **tx).await?;
    sqlx::query!("INSERT INTO rel_vorgang_init_person(initiator, vorgang_id) SELECT val, $2 FROM UNNEST($1::text[])as val;", 
    vg.initiator_personen.as_ref().map(|x|&x[..]), vg_id)
    .execute(&mut **tx).await?;

    // insert ids
    let ident_list = vg.ids.as_ref().map(|x|x.iter()
    .map(|el|el.id.clone()).collect::<Vec<_>>());
    let identt_list = vg.ids.as_ref().map(|x|x.iter()
    .map(|el|el.typ.to_string()).collect::<Vec<_>>());
    sqlx::query!("INSERT INTO rel_vorgang_ident (vorgang_id, typ, identifikator) 
    SELECT $1, id, ident.ident FROM 
    UNNEST($2::text[], $3::text[]) as ident(ident, typ)
    JOIN vg_ident_typ ON ident.typ = vg_ident_typ.api_key",
    vg_id, ident_list.as_ref().map(|x| &x[..]), identt_list.as_ref().map(|x| &x[..]))
    .execute(&mut **tx).await?;
    
    // insert stations
    for stat in &vg.stationen {
        insert_station(stat.clone(), vg_id, tx).await?;
    }
    tracing::info!("Insertion Successful with ID: {}", vg_id);
    Ok(vg_id)
}

pub async fn insert_station(
    stat: models::Station,
    vorgang_id: i32,
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<i32> {
    // master insert
    let stat_id = sqlx::query!(
        "INSERT INTO station (api_id, gremium, link, parl_id, titel, trojanergefahr, typ, zeitpunkt, vorgang_id)
        VALUES
        ($1, $2, $3, 
        (SELECT id FROM parlament WHERE api_key = $4), $5, $6, 
        (SELECT id FROM stationstyp WHERE api_key = $7), $8, $9)
        RETURNING station.id", 
        stat.api_id.unwrap_or(uuid::Uuid::now_v7()), stat.gremium, stat.link,
        stat.parlament.to_string(), stat.titel, stat.trojanergefahr.map(|x|x as i32), stat.typ.to_string(),
        stat.zeitpunkt, vorgang_id
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
        did.push(insert_dokument(dokument, tx).await?);
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
            doks.push(insert_dokument(stln.dokument, tx).await?);
        }
        sqlx::query!("INSERT INTO stellungnahme (stat_id, meinung, lobbyreg_link, dok_id)
        SELECT $1, mn, lobby, did FROM UNNEST($2::int4[], $3::text[], $4::int4[]) as blub(mn, lobby, did)",
        stat_id, &mng[..], &lobby as &[Option<String>], &doks[..]
        ).execute(&mut **tx).await?;
    }
    // schlagworte
    sqlx::query!("
        WITH existing_ids AS (SELECT DISTINCT id FROM schlagwort WHERE api_key = ANY($1::text[])),
        inserted AS(
            INSERT INTO schlagwort(api_key) 
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
fn sanitize_string(s: &str) -> String{
    s.to_string()
}
pub async fn insert_dokument(
    dok: models::Dokument,
    tx: &mut sqlx::PgTransaction<'_>) 
    -> Result<i32> {
    let did = sqlx::query!(
        "INSERT INTO dokument(api_id, titel, link, hash, last_mod, zusammenfassung, volltext, drucksnr, typ)
        VALUES(
            $1,$2,$3,$4,$5,$6,$7,$8,
            (SELECT id FROM dokumententyp WHERE api_key = $9)
        )RETURNING id", dok.api_id.unwrap_or(uuid::Uuid::now_v7()), dok.titel, dok.link,dok.hash,
        dok.letzte_modifikation,dok.zusammenfassung, dok.volltext, dok.drucksnr, dok.typ.to_string()
    ).map(|r|r.id).fetch_one(&mut **tx).await?;

    // Schlagworte
    sqlx::query!("
        WITH existing_ids AS (SELECT DISTINCT id FROM schlagwort WHERE api_key = ANY($1::text[])),
        inserted AS(
            INSERT INTO schlagwort(api_key) 
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
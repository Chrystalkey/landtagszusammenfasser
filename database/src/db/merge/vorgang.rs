#![allow(unused)]
use crate::error::DataValidationError;
/// Handles merging of two datasets.
/// in particular, stellungnahme & dokument are atomic.
/// station and vorgang are not in the sense that vorgang.stations and station.stellungnahmen are appendable and deletable.
/// This means the merge strategy is in general to:
/// 1. find a vorgang that is matching enough
///     a. if found exactly one, update the vorgang, see 2.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new vorgang, return
/// 2. if a., then update the vorgang properties
/// 3. for each station in the new vorgang, find a matching station
///     a. if found exactly one, update it, see 4.
///     b. if found more than one, send a message to the admins to select one
///     c. if found none, create a new station & insert
/// 4. if a., then update station properties
/// 5. for each stellungnahme in the new station, find a matching stellungnahme
///    a. if found exactly one, replace it
///    b. if found more than one, send a message to the admins to select one
///    c. if found none, create a new stellungnahme & insert

use crate::{LTZFServer, Result};
use crate::db::insert;
use openapi::models;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    NoMatch,
}

/// this function determines what means "matching enough".
/// 1. wenn api_id matcht
/// 2. wenn wp, typ und mindestens ein identifikator matchen
/// [TODO]3. wenn wp, typ und 1/5 des volltextes sich "sehr ähnlich sind (tbd)"
pub async fn vorgang_merge_candidates(
    model: &models::Vorgang,
    executor: impl sqlx::PgExecutor<'_>,
    srv: &LTZFServer,
) -> Result<MergeState<i32>> {
    let obj = "merged Vorgang";
    let ident_t: Vec<_> = model.ids.as_ref().unwrap_or(&vec![]).iter().map(|x|x.id.clone()).collect();
    let identt_t: Vec<_> = model.ids.as_ref().unwrap_or(&vec![]).iter().map(|x| srv.guard_ts(x.typ, model.api_id, obj).unwrap()).collect();
    let initds: Vec<_> = model.stationen.iter()
    .filter(|s| s.typ == models::Stationstyp::ParlInitiativ)
    .map(|s| 
        s.dokumente.iter().filter(|d| if let models::DokRef::Dokument(d) = d{
            d.typ == models::Doktyp::Drucksache && d.vorwort.is_some()
        }else{false})
        .map(|d|if let models::DokRef::Dokument(d) = d{d.vorwort.clone().unwrap()}else{unreachable!()})
        .map(|s|s.to_string())
    )
    .flatten()
    .collect();
    let result = sqlx::query!(
        "WITH db_id_table AS (
            SELECT rel_vg_ident.vg_id as vg_id, identifikator as ident, vg_ident_typ.value as idt_str
            FROM vg_ident_typ, rel_vg_ident 
            WHERE vg_ident_typ.id = rel_vg_ident.typ),
	initds_vwtable AS ( --vorworte von initiativdrucksachen von stationen
			SELECT s.vg_id, d.vorwort, d.volltext FROM dokument d
				INNER JOIN rel_station_dokument rsd ON rsd.dok_id=d.id
				INNER JOIN dokumententyp dt ON dt.id=d.typ
				INNER JOIN station s ON s.id = rsd.stat_id
				WHERE rsd.stat_id=s.id
				AND dt.value='drucksache'
		)

SELECT DISTINCT(vorgang.id), vorgang.api_id FROM vorgang -- gib vorgänge, bei denen
	INNER JOIN vorgangstyp vt ON vt.id = vorgang.typ
	WHERE
	vorgang.api_id = $1 OR -- entweder die API ID genau übereinstimmt (trivialer Fall) ODER
	(
	vorgang.wahlperiode = $4 AND -- wahlperiode und 
	vt.value = $5 AND            -- typ übereinstimmen und 
		(EXISTS (SELECT * FROM UNNEST($2::text[], $3::text[]) as eingabe(ident, typ), db_id_table WHERE  -- eine übereinstimmende ID existiert
			db_id_table.vg_id = vorgang.id AND
			eingabe.ident = db_id_table.ident AND
			eingabe.typ = db_id_table.idt_str)
		OR -- oder 
		EXISTS (SELECT * FROM UNNEST($6::text[]) eingabe(vw), initds_vwtable ids
		WHERE ids.vg_id = vorgang.id
		AND SIMILARITY(vw, ids.vorwort) > 0.8
		)
		)
	);", 
    model.api_id, &ident_t[..], &identt_t[..], model.wahlperiode as i32, 
    srv.guard_ts(model.typ, model.api_id, obj)?, &initds[..])
    .fetch_all(executor).await?;

    tracing::debug!("Found {} matches for Vorgang with api_id: {}",result.len(),model.api_id);

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch(result[0].id),
        _ => {
            tracing::warn!("Warning: Mehrere Vorgänge gefunden, die als Kandidaten für Merge infrage kommen für den Vorgang `{}`:\n{:?}", 
            model.api_id, result.iter().map(|r|r.api_id).collect::<Vec<_>>());
            MergeState::AmbiguousMatch(
                result.iter().map(|x|x.id).collect()
            )
        }
    })
}

/// bei gleichem Vorgang => Vorraussetzung
/// 1. wenn die api_id matcht
/// 2. wenn typ, parlament matcht und mindestens ein Dokument gleich ist
pub async fn station_merge_candidates(model: &models::Station, vorgang: i32, executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer)-> Result<MergeState<i32>> {
    let obj = "merged station";
    let api_id = model.api_id.unwrap_or(uuid::Uuid::now_v7());
    let dok_hash: Vec<_> = model.dokumente.iter()
    .filter(|x| match x{models::DokRef::Dokument(_)=>{true}, _ => false})
    .map(|x| {if let models::DokRef::Dokument(d) = x{d.hash.clone()}else{unreachable!()}})
    .collect();
    let result = 
    sqlx::query!("SELECT s.id, s.api_id FROM station s
    INNER JOIN stationstyp st ON st.id=s.typ
    WHERE s.api_id = $1 OR
    (s.vg_id = $2 AND st.value = $3 AND 
    EXISTS (SELECT * FROM rel_station_dokument rsd
	INNER JOIN dokument d ON rsd.dok_id=d.id
	WHERE rsd.stat_id = s.id
	AND d.hash IN (SELECT str FROM UNNEST($4::text[]) blub(str))
	)
	)", model.api_id, vorgang, srv.guard_ts(model.typ, api_id, obj)?, &dok_hash[..])
    .fetch_all(executor).await?;
    tracing::debug!("Found {} matches for Station with api_id: {}",result.len(), api_id);

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch(result[0].id),
        _ => {
            tracing::warn!("Warning: Mehrere Stationen gefunden, die als Kandidaten für Merge infrage kommen für Station `{}`:\n{:?}", 
            api_id, result.iter().map(|r|r.api_id).collect::<Vec<_>>());
            MergeState::AmbiguousMatch(
                result.iter().map(|x|x.id).collect()
            )
        }
    })
}
pub async fn dokument_merge_candidates(model: &models::Dokument, executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer) -> Result<MergeState<i32>> {
    todo!()
}

/// basic data items are to be overridden by newer information. 
/// Excempt from this is the api_id, since this is a permanent document identifier.
/// All 
pub async fn execute_merge_dokument (
    model: &models::Dokument,
    candidate: i32,
    tx:  &mut sqlx::PgTransaction<'_>, srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    // master update
    sqlx::query!("UPDATE dokument SET
    drucksnr = $2, titel =$3,
    kurztitel = COALESCE($4, kurztitel), vorwort=COALESCE($5, vorwort),
    volltext=COALESCE($6, volltext), zusammenfassung=COALESCE($7, zusammenfassung),
    last_mod=$8, link=$9, hash=$10
    WHERE dokument.id = $1
    ", db_id,
    model.drucksnr, model.titel,
    model.kurztitel, model.vorwort,
    model.volltext, model.zusammenfassung,
    model.letzte_modifikation, model.link, model.hash
    ).execute(&mut **tx).await?;
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

        INSERT INTO rel_dok_schlagwort(dok_id, sw_id)
        SELECT $2, allofthem.id FROM allofthem",
        model.schlagworte.as_ref().map(|x|&x[..]), db_id
    )
    .execute(&mut **tx).await?;
    // autoren
    sqlx::query!("INSERT INTO rel_dok_autor(dok_id, autor)
    SELECT $1, blub FROM UNNEST($2::text[]) as blub ON CONFLICT DO NOTHING", db_id,
    model.autoren.as_ref().map(|x|&x[..])).execute(&mut **tx).await?;
    sqlx::query!("INSERT INTO rel_dok_autorperson(dok_id, autor)
    SELECT $1, blub FROM UNNEST($2::text[]) as blub ON CONFLICT DO NOTHING", db_id,
    model.autorpersonen.as_ref().map(|x|&x[..])).execute(&mut **tx).await?;
    return Ok(());
}
pub async fn execute_merge_station (
    model: &models::Station,
    candidate: i32,
    tx: &mut sqlx::PgTransaction<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    let obj = "merge station";
    let sapi = sqlx::query!("SELECT api_id FROM station WHERE id = $1", db_id)
    .map(|x| x.api_id).fetch_one(&mut **tx).await?;
    // pre-master updates
    let gr_id = if let Some(gremium) = &model.gremium {
        let id = sqlx::query!("INSERT INTO gremium(name, parl) 
        VALUES ($1, (SELECT id FROM parlament WHERE value=$2)) 
        ON CONFLICT(name, parl) DO UPDATE SET name=$1 RETURNING id",
        gremium.name, gremium.parlament.to_string()).map(|r|r.id).fetch_one(&mut **tx).await?;
        Some(id)
    }else {
        None
    };
    // master update
    sqlx::query!("UPDATE station SET 
        gr_id = COALESCE($2, gr_id),
        p_id = (SELECT id FROM parlament WHERE value = $3),
        typ = (SELECT id FROM stationstyp WHERE value = $4),
        titel = COALESCE($5, titel), 
        start_zeitpunkt = $6, letztes_update = NOW(),
        trojanergefahr = COALESCE($7, trojanergefahr),
        link = COALESCE($8, link)
        WHERE station.id = $1", 
        db_id, gr_id, model.parlament.to_string(),
        srv.guard_ts(model.typ, sapi, obj)?,
        model.titel, model.start_zeitpunkt, model.trojanergefahr.map(|x| x as i32), model.link
        ).execute(&mut **tx).await?;
    // betroffene Texte
    sqlx::query!(
        "INSERT INTO rel_station_gesetz(stat_id, gesetz)
        SELECT $1, blub FROM UNNEST($2::text[]) as blub
        ON CONFLICT DO NOTHING",
        db_id, model.betroffene_texte.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx).await?;
    // schlagworte
    sqlx::query!("
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
    SELECT $2, allofthem.id FROM allofthem",
    model.schlagworte.as_ref().map(|x|&x[..]), db_id
    )
    .execute(&mut **tx).await?;
    // dokumente
    let mut insert_ids = vec![];
    for dok in model.dokumente.iter(){
        // if id & not in database: fail.
        // if id & in database: add to list of associated documents
        // if document: match & integrate or insert.
        match dok{
            models::DokRef::String(uuid) => {
                let uuid = uuid::Uuid::parse_str(uuid)?;
                let id = sqlx::query!("SELECT id FROM dokument d WHERE d.api_id = $1", uuid)
                .map(|r|r.id).fetch_optional(&mut **tx).await?;
                if id.is_none(){
                    return Err(DataValidationError::IncompleteDataSupplied { 
                        input: format!("Supplied uuid `{}` as document id for station `{}`, but no such ID is in the database.",
                        uuid, sapi) }.into());
                }
                insert_ids.push(id.unwrap());
            },
            models::DokRef::Dokument(dok) =>{
                let matches = dokument_merge_candidates(&*dok, &mut **tx, srv).await?;
                match matches{
                    MergeState::NoMatch => {
                        let did = crate::db::insert::insert_dokument((**dok).clone(), tx, srv).await?;
                        insert_ids.push(did);
                    },
                    MergeState::OneMatch(matchmod) => {
                        tracing::debug!("Found exactly one match with db id: {}. Merging...", matchmod);
                        execute_merge_dokument(&**dok,matchmod, tx, srv).await?;
                    }
                    MergeState::AmbiguousMatch(matche) => {todo!("Error out")}
                }
            }
        }
        sqlx::query!("INSERT INTO rel_station_dokument(stat_id, dok_id) 
        SELECT $1, did FROM UNNEST($2::int4[]) as did", db_id, &insert_ids[..])
        .execute(&mut **tx).await?;
    }
    // stellungnahmen
    todo!("TODO: You stopped here!")
}

pub async fn execute_merge_vorgang (
    model: &models::Vorgang,
    candidate: i32,
    executor: impl sqlx::PgExecutor<'_>,srv: &LTZFServer
) -> Result<()> {
    let db_id = candidate;
    todo!()
}

pub async fn run_integration(model: &models::Vorgang, server: &LTZFServer) -> Result<()> {
    let mut tx = server.sqlx_db.begin().await?;
    tracing::debug!(
        "Looking for Merge Candidates for Vorgang with api_id: {:?}",
        model.api_id);
    let candidates = vorgang_merge_candidates(model, &mut *tx, server).await?;
    match candidates {
        MergeState::NoMatch => {
            tracing::info!(
                "No Merge Candidate found, Inserting Complete Vorgang with api_id: {:?}",
                model.api_id
            );
            let model = model.clone();
            insert::insert_vorgang(&model, &mut tx, server).await?;
        }
        MergeState::OneMatch(one) => {
            let api_id = sqlx::query!("SELECT api_id FROM vorgang WHERE id = $1", one)
            .map(|r|r.api_id).fetch_one(&mut *tx).await?;
            tracing::info!(
                "Matching Vorgang in the DB has api_id: {}, Updating with data from: {}",
                api_id,
                model.api_id
            );
            let model = model.clone();
            execute_merge_vorgang(&model, one, &mut *tx, server).await?;
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!("Ambiguous matches for Vorgang with api_id: {:?}", model.api_id);
            tracing::debug!("Details:  {:?} \n\n {:?}", model, many);
            unimplemented!("Notify Admins via $WAY");
        }
    }
    tx.commit().await?;
    Ok(())
}

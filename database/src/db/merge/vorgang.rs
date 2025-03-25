#![allow(unused)]
use super::MergeState;
use crate::db::insert::{self, insert_or_retrieve_autor};
use crate::error::DataValidationError;
use crate::utils::notify::notify_ambiguous_match;
/// Handles merging of two datasets.
/// vorgang, station and dokument are mergeable, meaning their data is not atomic.
/// Stellungnahme is handled like dokument with the rest being overridable data points
/// API_ID or other uniquely identifying information is not overridden, but preserved.
/// array-like structures are merged by a modified union operation:
/// for each element:
///     - if it is mergeable and one merge candidate found, merge
///     - if it is not mergeable and has a match in the existing set, the existing element takes precedence and is not replaced
///     - if it is not mergeable and has no match it is added to the set.
use crate::{LTZFServer, Result};
use openapi::models;

/// this function determines what means "matching enough".
/// 1. wenn api_id matcht
/// 2. wenn wp, typ und mindestens ein identifikator matchen
/// 3. wenn wp, typ und das vorwort sich "sehr ähnlich sind (>0.8)"
pub async fn vorgang_merge_candidates(
    model: &models::Vorgang,
    executor: impl sqlx::PgExecutor<'_>,
    srv: &LTZFServer,
) -> Result<MergeState<i32>> {
    let obj = "merged Vorgang";
    let ident_t: Vec<_> = model
        .ids
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(|x| x.id.clone())
        .collect();
    let identt_t: Vec<_> = model
        .ids
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(|x| srv.guard_ts(x.typ, model.api_id, obj).unwrap())
        .collect();
    let initds: Vec<_> = model
        .stationen
        .iter()
        .filter(|&s| s.typ == models::Stationstyp::ParlInitiativ)
        .map(|s| {
            s.dokumente
                .iter()
                .filter(|&d| {
                    if let models::DokRef::Dokument(d) = d {
                        d.typ == models::Doktyp::Entwurf && d.vorwort.is_some()
                    } else {
                        false
                    }
                })
                .map(|d| {
                    if let models::DokRef::Dokument(d) = d {
                        d.vorwort.clone().unwrap()
                    } else {
                        unreachable!()
                    }
                })
                .map(|s| s.to_string())
        })
        .flatten()
        .collect();
    let result = sqlx::query!(
        "WITH db_id_table AS (
            SELECT rel_vorgang_ident.vg_id as vg_id, identifikator as ident, vg_ident_typ.value as idt_str
            FROM vg_ident_typ, rel_vorgang_ident 
            WHERE vg_ident_typ.id = rel_vorgang_ident.typ),
	initds_vwtable AS ( --vorworte von initiativdrucksachen von stationen
			SELECT s.vg_id, d.vorwort, d.volltext FROM dokument d
				INNER JOIN rel_station_dokument rsd ON rsd.dok_id=d.id
				INNER JOIN dokumententyp dt ON dt.id=d.typ
				INNER JOIN station s ON s.id = rsd.stat_id
				WHERE rsd.stat_id=s.id
				AND (dt.value='entwurf' OR dt.value = 'preparl-entwurf')
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

    tracing::debug!(
        "Found {} matches for Vorgang with api_id: {}",
        result.len(),
        model.api_id
    );

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch(result[0].id),
        _ => {
            tracing::warn!("Warning: Mehrere Vorgänge gefunden, die als Kandidaten für Merge infrage kommen für den Vorgang `{}`:\n{:?}",
            model.api_id, result.iter().map(|r|r.api_id).collect::<Vec<_>>());
            MergeState::AmbiguousMatch(result.iter().map(|x| x.id).collect())
        }
    })
}

/// bei gleichem Vorgang => Vorraussetzung
/// 1. wenn die api_id matcht
/// 2. wenn typ, parlament, gremium matcht und mindestens ein Dokument gleich ist
pub async fn station_merge_candidates(
    model: &models::Station,
    vorgang: i32,
    executor: impl sqlx::PgExecutor<'_>,
    srv: &LTZFServer,
) -> Result<MergeState<i32>> {
    let obj = "merged station";
    let api_id = model.api_id.unwrap_or(uuid::Uuid::now_v7());
    let dok_hash: Vec<_> = model
        .dokumente
        .iter()
        .filter(|x| match x {
            models::DokRef::Dokument(_) => true,
            _ => false,
        })
        .map(|x| {
            if let models::DokRef::Dokument(d) = x {
                d.hash.clone()
            } else {
                unreachable!()
            }
        })
        .collect();
    let (gr_name, gr_wp, gr_parl) = if let Some(gremium) = &model.gremium {
        (Some(gremium.name.clone()), Some(gremium.wahlperiode as i32), Some(gremium.parlament.to_string()))
    } else {
        (None, None, None)
    };
    let result = sqlx::query!(
        "SELECT s.id, s.api_id FROM station s
    INNER JOIN stationstyp st ON st.id=s.typ
    LEFT JOIN gremium g ON g.id=s.gr_id
    LEFT JOIN parlament p ON p.id = g.parl
    WHERE s.api_id = $1 OR
    (s.vg_id = $2 AND st.value = $3 AND  -- vorgang und stationstyp übereinstimmen
    (g.name = $4 OR $4 IS NULL) AND  -- gremiumname übereinstimmt
    (p.value = $5 OR $5 IS NULL) AND  -- parlamentname übereinstimmt
    (g.wp = $6 OR $6 IS NULL) AND -- gremium wahlperiode übereinstimmt
    EXISTS (SELECT * FROM rel_station_dokument rsd
        INNER JOIN dokument d ON rsd.dok_id=d.id
        WHERE rsd.stat_id = s.id
        AND d.hash IN (SELECT str FROM UNNEST($7::text[]) blub(str))
	))",
        model.api_id,
        vorgang,
        srv.guard_ts(model.typ, api_id, obj)?,
        gr_name, gr_parl, gr_wp,
        &dok_hash[..]
    )
    .fetch_all(executor)
    .await?;
    tracing::debug!(
        "Found {} matches for Station with api_id: {}",
        result.len(),
        api_id
    );

    Ok(match result.len() {
        0 => MergeState::NoMatch,
        1 => MergeState::OneMatch(result[0].id),
        _ => {
            tracing::warn!("Warning: Mehrere Stationen gefunden, die als Kandidaten für Merge infrage kommen für Station `{}`:\n{:?}",
            api_id, result.iter().map(|r|r.api_id).collect::<Vec<_>>());
            MergeState::AmbiguousMatch(result.iter().map(|x| x.id).collect())
        }
    })
}
/// bei gleichem
/// - hash oder api_id oder drucksnr
pub async fn dokument_merge_candidates(
    model: &models::Dokument,
    executor: impl sqlx::PgExecutor<'_>,
    srv: &LTZFServer,
) -> Result<MergeState<i32>> {
    let dids = sqlx::query!(
        "SELECT d.id FROM dokument d WHERE 
        d.hash = $1 OR
        d.api_id = $2 OR
        d.drucksnr = $3",
        model.hash,
        model.api_id,
        model.drucksnr
    )
    .map(|r| r.id)
    .fetch_all(executor)
    .await?;
    if dids.is_empty() {
        return Ok(MergeState::NoMatch);
    } else if dids.len() == 1 {
        return Ok(MergeState::OneMatch(dids[0]));
    } else {
        return Ok(MergeState::AmbiguousMatch(dids));
    }
}

/// basic data items are to be overridden by newer information.
/// Excempt from this is the api_id, since this is a permanent document identifier.
/// All
pub async fn execute_merge_dokument(
    model: &models::Dokument,
    candidate: i32,
    tx: &mut sqlx::PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<()> {
    let db_id = candidate;
    // master update
    sqlx::query!(
        "UPDATE dokument SET
        drucksnr = $2, titel =$3,
        kurztitel = COALESCE($4, kurztitel), vorwort=COALESCE($5, vorwort),
        volltext=COALESCE($6, volltext), zusammenfassung=COALESCE($7, zusammenfassung),
        zp_lastmod=$8, link=$9, hash=$10, meinung=$11
        WHERE dokument.id = $1
        ",
        db_id,
        model.drucksnr,
        model.titel,
        model.kurztitel,
        model.vorwort,
        model.volltext,
        model.zusammenfassung,
        model.zp_modifiziert,
        model.link,
        model.hash,
        model.meinung.map(|x|x as i32)
    )
    .execute(&mut **tx)
    .await?;
    // schlagworte::UNION
    insert::insert_dok_sw(db_id, model.schlagworte.clone().unwrap_or(vec![]), tx).await?;
    // autoren::UNION
    let mut aids = vec![];
    for a in &model.autoren{
        aids.push(insert_or_retrieve_autor(a, tx, srv).await?);
    }
    sqlx::query!(
        "INSERT INTO rel_dok_autor(dok_id, aut_id)
    SELECT $1, blub FROM UNNEST($2::int4[]) as blub 
    ON CONFLICT DO NOTHING",
        db_id, &aids[..])
    .execute(&mut **tx)
    .await?;

    tracing::info!("Merging Dokument into Database successful");
    return Ok(());
}

pub async fn execute_merge_station(
    model: &models::Station,
    candidate: i32,
    tx: &mut sqlx::PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<()> {
    let db_id = candidate;
    let obj = "merge station";
    let sapi = sqlx::query!("SELECT api_id FROM station WHERE id = $1", db_id)
        .map(|x| x.api_id)
        .fetch_one(&mut **tx)
        .await?;
    // pre-master updates
    let gr_id = if let Some(gremium) = &model.gremium {
        let id = insert::insert_or_retrieve_gremium(gremium, tx, srv).await?;
        Some(id)
    } else {
        None
    };
    // master update
    sqlx::query!(
        "UPDATE station SET 
        gr_id = COALESCE($2, gr_id),
        p_id = (SELECT id FROM parlament WHERE value = $3),
        typ = (SELECT id FROM stationstyp WHERE value = $4),
        titel = COALESCE($5, titel),
        zp_start = $6, zp_modifiziert = COALESCE($7, NOW()),
        trojanergefahr = COALESCE($8, trojanergefahr),
        link = COALESCE($9, link),
        gremium_isff = $10
        WHERE station.id = $1",
        db_id,
        gr_id,
        model.parlament.to_string(),
        srv.guard_ts(model.typ, sapi, obj)?,
        model.titel,
        model.zp_start,
        model.zp_modifiziert,
        model.trojanergefahr.map(|x| x as i32),
        model.link,
        model.gremium_federf
    )
    .execute(&mut **tx)
    .await?;

    // links::UNION
    sqlx::query!(
        "INSERT INTO rel_station_link(stat_id, link)
        SELECT $1, blub FROM UNNEST($2::text[]) as blub
        ON CONFLICT DO NOTHING",
        db_id,
        model.additional_links.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx)
    .await?;

    // schlagworte::UNION
    insert::insert_station_sw(db_id, model.schlagworte.clone().unwrap_or(vec![]), tx).await?;
    
    // dokumente::UNION
    let mut insert_ids = vec![];
    for dok in model.dokumente.iter() {
        // if id & not in database: fail.
        // if id & in database: add to list of associated documents
        // if document: match & integrate or insert.
        match dok {
            models::DokRef::String(uuid) => {
                let uuid = uuid::Uuid::parse_str(uuid)?;
                let id = sqlx::query!("SELECT id FROM dokument d WHERE d.api_id = $1", uuid)
                    .map(|r| r.id)
                    .fetch_optional(&mut **tx)
                    .await?;
                if id.is_none() {
                    return Err(DataValidationError::IncompleteDataSupplied {
                        input: format!("Supplied uuid `{}` as document id for station `{}`, but no such ID is in the database.",
                        uuid, sapi) }.into());
                }
                insert_ids.push(id.unwrap());
            }
            models::DokRef::Dokument(dok) => {
                let matches = dokument_merge_candidates(&*dok, &mut **tx, srv).await?;
                match matches {
                    MergeState::NoMatch => {
                        let did =
                            crate::db::insert::insert_dokument((**dok).clone(), tx, srv).await?;
                        insert_ids.push(did);
                    }
                    MergeState::OneMatch(matchmod) => {
                        tracing::debug!(
                            "Found exactly one match with db id: {}. Merging...",
                            matchmod
                        );
                        execute_merge_dokument(&**dok, matchmod, tx, srv).await?;
                    }
                    MergeState::AmbiguousMatch(matches) => {
                        let api_ids = sqlx::query!(
                            "SELECT api_id FROM dokument WHERE id = ANY($1::int4[])",
                            &matches[..]
                        )
                        .map(|r| r.api_id)
                        .fetch_all(&mut **tx)
                        .await?;
                        notify_ambiguous_match(
                            api_ids,
                            &**dok,
                            "execute merge station.dokumente",
                            srv,
                        )?;
                        return Err(DataValidationError::AmbiguousMatch {
                            message: format!("Ambiguous document match(station), see notification"),
                        }
                        .into());
                    }
                }
            }
        }
        sqlx::query!(
            "INSERT INTO rel_station_dokument(stat_id, dok_id) 
        SELECT $1, did FROM UNNEST($2::int4[]) as did",
            db_id,
            &insert_ids[..]
        )
        .execute(&mut **tx)
        .await?;
    }
    
    // stellungnahmen
    for stln in model.stellungnahmen.as_ref().unwrap_or(&vec![]) {
        match dokument_merge_candidates(stln, &mut **tx, srv).await? {
            MergeState::NoMatch => {
                let did = insert::insert_dokument(stln.clone(), tx, srv).await?;
                sqlx::query!(
                    "INSERT INTO rel_station_stln(stat_id, dok_id) VALUES($1, $2);",
                    db_id,
                    did
                )
                .execute(&mut **tx)
                .await?;
            }
            MergeState::OneMatch(did) => {
                execute_merge_dokument(&stln, did, tx, srv).await?;
            }
            MergeState::AmbiguousMatch(matches) => {
                let api_ids = sqlx::query!(
                    "SELECT api_id FROM dokument WHERE id = ANY($1::int4[])",
                    &matches[..]
                )
                .map(|r| r.api_id)
                .fetch_all(&mut **tx)
                .await?;
                notify_ambiguous_match(api_ids, stln, "execute merge station.stellungnahmen", srv)?;
                return Err(DataValidationError::AmbiguousMatch {
                    message: format!("Ambiguous document match(Stln), see notification"),
                }
                .into());
            }
        };
    }
    tracing::info!("Merging Station into Database successful");
    Ok(())
}

pub async fn execute_merge_vorgang(
    model: &models::Vorgang,
    candidate: i32,
    tx: &mut sqlx::PgTransaction<'_>,
    srv: &LTZFServer,
) -> Result<()> {
    let db_id = candidate;
    let obj = "Vorgang";
    let vapi = model.api_id;
    /// master insert
    sqlx::query!(
        "UPDATE vorgang SET
        titel = $1, kurztitel = $2,
        verfaend = $3, wahlperiode = $4,
        typ = (SELECT id FROM vorgangstyp WHERE value = $5)
        WHERE vorgang.id = $6",
        model.titel,
        model.kurztitel,
        model.verfassungsaendernd,
        model.wahlperiode as i32,
        srv.guard_ts(model.typ, vapi, obj)?,
        db_id
    )
    .execute(&mut **tx)
    .await?;
    /// initiatoren / initpersonen::UNION
    let mut aids = vec![];
    for a in &model.initiatoren{
        aids.push(insert_or_retrieve_autor(a, tx, srv).await?);
    }
    sqlx::query!(
        "INSERT INTO rel_vorgang_init (vg_id, in_id)
        SELECT $1, blub FROM UNNEST($2::int4[]) as blub
        ON CONFLICT DO NOTHING",
        db_id,
        &aids[..]
    )
    .execute(&mut **tx)
    .await?;
    /// links
    let links = model.links.clone().unwrap_or(vec![]);
    sqlx::query!(
        "INSERT INTO rel_vorgang_links (vg_id, link)
        SELECT $1, blub FROM UNNEST($2::text[]) as blub
        ON CONFLICT DO NOTHING",
        db_id,
        &links[..]
    )
    .execute(&mut **tx)
    .await?;
    /// identifikatoren
    let ident_list = model
        .ids
        .as_ref()
        .map(|x| x.iter().map(|el| el.id.clone()).collect::<Vec<_>>());

    let identt_list = model.ids.as_ref().map(|x| {
        x.iter()
            .map(|el| srv.guard_ts(el.typ, model.api_id, obj).unwrap())
            .collect::<Vec<_>>()
    });

    sqlx::query!(
        "INSERT INTO rel_vorgang_ident (vg_id, typ, identifikator)
        SELECT $1, vit.id, ident FROM 
        UNNEST($2::text[], $3::text[]) blub(typ_value, ident)
        INNER JOIN vg_ident_typ vit ON vit.value = typ_value
        ON CONFLICT DO NOTHING
        ",
        db_id,
        identt_list.as_ref().map(|x| &x[..]),
        ident_list.as_ref().map(|x| &x[..])
    )
    .execute(&mut **tx)
    .await?;

    for stat in &model.stationen {
        match station_merge_candidates(stat, db_id, &mut **tx, srv).await? {
            MergeState::NoMatch => {
                insert::insert_station(stat.clone(), db_id, tx, srv).await?;
            }
            MergeState::OneMatch(merge_station) => {
                execute_merge_station(stat, db_id, tx, srv).await?
            }
            MergeState::AmbiguousMatch(matches) => {
                let mids = sqlx::query!(
                    "SELECT api_id FROM station WHERE id = ANY($1::int4[]);",
                    &matches[..]
                )
                .map(|r| r.api_id)
                .fetch_all(&mut **tx)
                .await?;
                notify_ambiguous_match(mids, stat, 
                    "exec_merge_vorgang: station matching", srv);
            }
        }
    }

    tracing::info!(
        "Merging of Vg Successful: Merged `{}`(ext) with  `{}`(db)",
        model.api_id,
        vapi
    );
    Ok(())
}

pub async fn run_integration(model: &models::Vorgang, server: &LTZFServer) -> Result<()> {
    let mut tx = server.sqlx_db.begin().await?;
    tracing::debug!(
        "Looking for Merge Candidates for Vorgang with api_id: {:?}",
        model.api_id
    );
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
                .map(|r| r.api_id)
                .fetch_one(&mut *tx)
                .await?;
            tracing::info!(
                "Matching Vorgang in the DB has api_id: {}, Updating with data from: {}",
                api_id,
                model.api_id
            );
            let model = model.clone();
            execute_merge_vorgang(&model, one, &mut tx, server).await?;
        }
        MergeState::AmbiguousMatch(many) => {
            tracing::warn!(
                "Ambiguous matches for Vorgang with api_id: {:?}",
                model.api_id
            );
            tracing::debug!("Details:  {:?} \n\n {:?}", model, many);
            unimplemented!("Notify Admins via $WAY");
        }
    }
    tx.commit().await?;
    Ok(())
}
mod scenariotest {
    #![cfg(test)]
    use crate::LTZFServer;
    use futures::FutureExt;
    use similar::ChangeTag;
    use std::collections::HashSet;
    use std::panic::AssertUnwindSafe;

    use openapi::models::{self, VorgangGetHeaderParams, VorgangGetQueryParams};
    use serde::Deserialize;

    #[allow(unused)]
    use tracing::{debug, error, info, warn};

    fn xor(one: bool, two: bool) -> bool {
        return (one && two) || (!one && !two);
    }
    #[allow(unused)]
    struct TestScenario<'obj> {
        name: &'obj str,
        context: Vec<models::Vorgang>,
        vorgang: models::Vorgang,
        result: Vec<models::Vorgang>,
        shouldfail: bool,
        server: LTZFServer,
        span: tracing::Span,
    }
    #[derive(Deserialize)]
    struct PTS {
        context: Vec<models::Vorgang>,
        vorgang: models::Vorgang,
        result: Vec<models::Vorgang>,
        #[serde(default = "default_bool")]
        shouldfail: bool,
    }
    fn default_bool() -> bool {
        false
    }
    impl<'obj> TestScenario<'obj> {
        async fn new(path: &'obj std::path::Path, server: &LTZFServer) -> Self {
            let name = path.file_stem().unwrap().to_str().unwrap();
            info!("Creating Merge Test Scenario with name: {}", name);
            let span = tracing::span!(tracing::Level::INFO, "Mergetest", name = name);
            let dropquery = format!("DROP DATABASE IF EXISTS \"testing_{}\" WITH (FORCE);", name);
            let query = format!(
                "CREATE DATABASE \"testing_{}\" WITH OWNER 'ltzf-user';",
                name
            );
            sqlx::query(&dropquery)
                .execute(&server.sqlx_db)
                .await
                .unwrap();
            sqlx::query(&query).execute(&server.sqlx_db).await.unwrap();
            let test_db_url = std::env::var("DATABASE_URL")
                .unwrap()
                .replace("5432/ltzf", &format!("5432/testing_{}", name));
            let pts: PTS = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
            let server = LTZFServer {
                config: crate::Configuration {
                    ..Default::default()
                },
                mailbundle: None,
                sqlx_db: sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&test_db_url)
                    .await
                    .unwrap(),
            };
            sqlx::migrate!().run(&server.sqlx_db).await.unwrap();
            for vorgang in pts.context.iter() {
                crate::db::merge::vorgang::run_integration(vorgang, &server)
                    .await
                    .unwrap()
            }
            Self {
                name,
                context: pts.context,
                vorgang: pts.vorgang,
                result: pts.result,
                shouldfail: pts.shouldfail,
                span,
                server,
            }
        }
        async fn push(&self) {
            info!("Running main Merge test");
            crate::db::merge::vorgang::run_integration(&self.vorgang, &self.server)
                .await
                .unwrap();
        }
        async fn check(&self) {
            info!("Checking for Correctness");
            let paramock = VorgangGetQueryParams {
                vgtyp: None,
                wp: None,
                inipsn: None,
                iniorg: None,
                inifch: None,
                p: None,
                since: None,
                until: None,
                limit: None,
                offset: None,
            };
            let hdmock = VorgangGetHeaderParams{
                if_modified_since: None
            };
            let mut tx = self.server.sqlx_db.begin().await.unwrap();
            let db_vorgangs = crate::db::retrieve::vorgang_by_parameter(&paramock, &hdmock, &mut tx)
                .await
                .unwrap();

            tx.rollback().await.unwrap();
            for expected in self.result.iter() {
                let mut found = false;
                for db_out in db_vorgangs.iter() {
                    if db_out == expected {
                        found = true;
                        break;
                    } else if xor(db_out.api_id != expected.api_id, self.shouldfail) {
                        std::fs::write(
                            format!("tests/{}_dumpa.json", self.name),
                            dump_objects(&expected, &db_out),
                        )
                        .unwrap();
                        assert!(
                            false,
                            "Differing object have the same api id: `{}`. Difference:\n{}",
                            db_out.api_id,
                            crate::db::merge::display_strdiff(
                                &serde_json::to_string_pretty(expected).unwrap(),
                                &serde_json::to_string_pretty(db_out).unwrap()
                            )
                        );
                    }
                }
                if xor(found, self.shouldfail) {
                    std::fs::write(
                        format!("tests/{}_dump.json", self.name),
                        serde_json::to_string_pretty(expected).unwrap(),
                    )
                    .unwrap();
                }
                assert!(found,
                    "({}): Expected to find Vorgang with api_id `{}`, but was not present in the output set, which contained: {:?}.\n\nDetails(Output Set):\n{:#?}",
                    self.name,
                    expected.api_id,
                    self.result.iter().map(|e|e.api_id).collect::<Vec<uuid::Uuid>>(),
                    db_vorgangs.iter().map(|v|
                    {println!("{}", serde_json::to_string_pretty(v).unwrap());""})
                    .collect::<Vec<_>>()
                );
            }

            assert!(self.result.len()==db_vorgangs.len(),
            "({}): Mismatch between the length of the expected set and the output set: {} (e) vs {} (o)\nOutput Set: {:#?}",
            self.name, self.result.len(), db_vorgangs.len(), db_vorgangs);
        }
        async fn run(self) {
            self.push().await;
            self.check().await;
        }
    }

    #[tokio::test]
    async fn test_merge_scenarios() {
        // set up database connection and clear it
        info!("Setting up Test Database Connection");
        let test_db_url = std::env::var("DATABASE_URL").unwrap();
        let master_server = LTZFServer {
            config: crate::Configuration {
                ..Default::default()
            },
            mailbundle: None,
            sqlx_db: sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&test_db_url)
                .await
                .unwrap(),
        };

        for path in std::fs::read_dir("tests/testfiles").unwrap() {
            if let Ok(path) = path {
                info!("Executing Scenario: {}", path.path().display());
                let ptb = path.path();
                let name = ptb.file_stem().unwrap().to_str().unwrap();

                let mut shouldfail = false;
                let scenario = TestScenario::new(&ptb, &master_server).await;
                let result = AssertUnwindSafe(async {
                    shouldfail = scenario.shouldfail;
                    scenario.run().await
                })
                .catch_unwind()
                .await;

                if result.is_ok() == shouldfail {
                    assert!(
                        false,
                        "The Scenario {} did not behave as expected: {}",
                        name,
                        if shouldfail {
                            "Succeeded, but should fail"
                        } else {
                            "Failed but should succeed"
                        }
                    );
                } else {
                    let query = format!("DROP DATABASE testing_{}", name);
                    sqlx::query(&query)
                        .execute(&master_server.sqlx_db)
                        .await
                        .unwrap();
                }
            } else {
                error!("Error: {:?}", path.unwrap_err())
            }
        }
    }
    fn dump_objects<T: serde::Serialize, S: serde::Serialize>(expected: &T, actual: &S) -> String {
        format!(
            "{{ \"expected-object\" : {},\n\"actual-object\" : {}}}",
            serde_json::to_string_pretty(expected).unwrap(),
            serde_json::to_string_pretty(actual).unwrap()
        )
    }
}

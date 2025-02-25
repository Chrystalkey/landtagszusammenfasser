use std::collections::HashMap;

use openapi::models;
use crate::Result;
use diesel::prelude::*;
use super::schema;

mod db_models{
    use diesel::prelude::*;

    pub struct ID{
        pub id: i32
    }
    impl QueryableByName<diesel::pg::Pg> for ID{
        fn build<'a>(row: &impl diesel::row::NamedRow<'a, diesel::pg::Pg>) -> diesel::deserialize::Result<Self> {
            Ok(
                ID { id: 
                    diesel::row::NamedRow::get(row, "id")?
                 }
            )
        }
    }
}
/// Inserts a new GSVH into the database.
pub fn insert_vorgang(
    api_vorgang: &models::Vorgang,
    connection: &mut diesel::PgConnection
) -> Result<i32> {
    tracing::info!("Inserting complete GSVH into the database");
    use schema::vorgang::dsl;
    use schema::vorgangstyp::dsl as typ_dsl;
    
    // master insert
    let vorgang_id = 
    diesel::insert_into(schema::vorgang::table)
    .values(
        (
            dsl::api_id.eq(api_vorgang.api_id),
            dsl::titel.eq(&api_vorgang.titel),
            dsl::verfaend.eq(api_vorgang.verfassungsaendernd),
            dsl::wahlperiode.eq(api_vorgang.wahlperiode as i32),
            dsl::typ.eq(
                typ_dsl::vorgangstyp
                .select(typ_dsl::id)
                .filter(typ_dsl::api_key.eq(&api_vorgang.typ.to_string()))
                .first::<i32>(connection)?
            ),
        )
    )
    .returning(dsl::id)
    .get_result::<i32>(connection)?;

    // insert links
    if let Some(links) = &api_vorgang.links {
        use schema::rel_vorgang_links::dsl as dsl;
        diesel::insert_into(schema::rel_vorgang_links::table)
        .values(
            links.iter()
            .cloned()
            .map(|s|(
                dsl::link.eq(s),
                dsl::vorgang_id.eq(vorgang_id)
            )
            )
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }

    // insert initiatoren
    if !api_vorgang.initiatoren.is_empty() {
        use schema::rel_vorgang_init::dsl as dsl;
        diesel::insert_into(schema::rel_vorgang_init::table)
        .values(
            api_vorgang.initiatoren.iter()
            .map(|s|
                (dsl::initiator.eq(s),
                dsl::vorgang_id.eq(vorgang_id)
            ))
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }

    if let Some(init_personen) = api_vorgang.initiator_personen.as_ref() {
        diesel::insert_into(schema::rel_vorgang_init_person::table)
        .values(
            init_personen.iter()
            .map(|s|
                (
                    schema::rel_vorgang_init_person::vorgang_id.eq(vorgang_id),
                    schema::rel_vorgang_init_person::initiator.eq(s.clone())
                )
            ).collect::<Vec<_>>()
        )
        .execute(connection)?;
    }

    // insert ids
    if let Some(ids) = api_vorgang.ids.as_ref() {
        use schema::rel_vorgang_id::dsl as dsl;
        let mut value_vec = vec![];

        for id_entry in ids.iter(){
            let value= (
                dsl::vorgang_id.eq(vorgang_id),
                dsl::identifikator.eq(&id_entry.id),
                dsl::typ.eq(
                    schema::identifikatortyp::table
                    .select(schema::identifikatortyp::id)
                    .filter(schema::identifikatortyp::api_key
                        .eq(&id_entry.typ.to_string())
                    )
                    .first::<i32>(connection)?
                )
            );
            value_vec.push(value);
        }
        diesel::insert_into(schema::rel_vorgang_id::table)
        .values(&value_vec)
        .execute(connection)?;
    }
    
    // insert statinons
    if !api_vorgang.stationen.is_empty() {
        for stat in api_vorgang.stationen.clone() {
            insert_station(stat, vorgang_id, connection)?;
        }
    }
    tracing::info!("Insertion Successful with ID: {}", vorgang_id);
    Ok(vorgang_id)
}

pub fn insert_station(
    stat: models::Station,
    vorgang_id: i32,
    connection: &mut diesel::PgConnection,
) -> Result<i32> {
    use schema::station::dsl;
    // master insert
    let stat_id = diesel::insert_into(schema::station::table)
    .values(
        (dsl::vorgang_id.eq(vorgang_id),
        dsl::titel.eq(stat.titel.clone()),
        dsl::trojanergefahr.eq(stat.trojanergefahr.map(|x| x as i32)),
        dsl::zeitpunkt.eq(stat.zeitpunkt),
        dsl::parl_id.eq(
            schema::parlament::table.select(schema::parlament::id)
            .filter(schema::parlament::api_key.eq(&stat.parlament.to_string()))
            .first::<i32>(connection)?
        ),
        dsl::typ.eq(
            schema::stationstyp::table.select(schema::stationstyp::id)
            .filter(schema::stationstyp::api_key.eq(&stat.typ.to_string()))
            .first::<i32>(connection)?
        ),
        dsl::link.eq(stat.link),
     )
    )
    .returning(dsl::id)
    .get_result::<i32>(connection)?;
    // ausschusssitzungen wenn anwendbar
    if  stat.typ != models::Stationstyp::ParlAusschber &&
        stat.ausschusssitzungen != None {
        let string = format!("Ausschussitzungen in Station mit anderem Typ `{}`", stat.typ.to_string());
        tracing::warn!("{}", &string);
        return Err(crate::error::LTZFError::Validation { source: crate::error::DataValidationError::InvalidEnumValue { msg: string } });
    }
    if let Some(auss) = stat.ausschusssitzungen {
        let string = 
        "Ausschussitzungen werden über den Endpoint /ausschusssitzungen hinzugefügt. 
        Die Daten der Ausschusssitzungen hier werden ignoriert";
        tracing::warn!(string);

        for ass in auss {
            // find a potentially corresponding AS in the db.
            let stmt_rs = diesel::sql_query(
                "SELECT ausschusssitzung.id FROM ausschusssitzung, ausschuss
WHERE
ausschuss.id = ausschusssitzung.as_id
AND termin BETWEEN $1 AND $2
AND SIMILARITY(ausschuss.name, $3) > 0.6
ORDER BY SIMILARITY(ausschuss.name, $3) DESC
LIMIT 1;")
            .bind::<diesel::sql_types::Timestamptz, _>(ass.termin.checked_sub_signed(chrono::TimeDelta::hours(6)).unwrap())
            .bind::<diesel::sql_types::Timestamptz, _>(ass.termin.checked_add_signed(chrono::TimeDelta::hours(6)).unwrap())
            .bind::<diesel::sql_types::VarChar, _>(ass.ausschuss.name)
            .get_result::<db_models::ID>(connection)
            .optional()?;
            if let Some(asid) = stmt_rs{
                let asid = asid.id;
                todo!()
                // set the tops there to reference this vorgang
            }
        }
    }

    // betroffene gesetzestexte
    if let Some(bt) = stat.betroffene_texte {
        if ! bt.is_empty(){
            diesel::insert_into(schema::rel_station_gesetz::table)
            .values(
                bt.iter().map(|gesetz|
                    (schema::rel_station_gesetz::stat_id.eq(stat_id),
                    schema::rel_station_gesetz::gesetz.eq(gesetz.clone())
                    )
                ).collect::<Vec<_>>()
            ).execute(connection)?;
        }
    }
    // assoziierte dokumente
    if !stat.dokumente.is_empty() {
        let mut dok_ids = vec![];
        for dok in stat.dokumente.clone(){
            dok_ids.push(insert_dokument(dok, connection)?);
        }
        
        diesel::insert_into(schema::rel_station_dokument::table)
        .values(
            dok_ids.iter()
            .map(|dok_id|
                (
                    schema::rel_station_dokument::stat_id.eq(stat_id),
                    schema::rel_station_dokument::dok_id.eq(*dok_id)
                )
            )
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }
    // stellungnahmen
    if let Some(stln) = stat.stellungnahmen {
        use schema::stellungnahme::dsl;
        for stln in stln {
            diesel::insert_into(schema::stellungnahme::table)
            .values( 
                (
                    dsl::meinung.eq(stln.meinung as i32),
                    dsl::lobbyreg_link.eq(stln.lobbyregister_link),
                    dsl::stat_id.eq(stat_id),
                    dsl::dok_id.eq(
                        insert_dokument(stln.dokument, connection)?
                    )
                )
            )
            .execute(connection)?;
        }
    }
    // schlagworte
    if let Some(sw) = stat.schlagworte {
        diesel::insert_into(schema::schlagwort::table)
        .values(
            sw.iter()
            .map(|s|
                schema::schlagwort::api_key.eq(s))
            .collect::<Vec<_>>()
        )
        .on_conflict_do_nothing()
        .execute(connection)?;
        let idvec : HashMap<String, i32> = 
        schema::schlagwort::table
        .filter(schema::schlagwort::api_key.eq_any(&sw))
        .select((schema::schlagwort::api_key, schema::schlagwort::id))
        .get_results::<(String, i32)>(connection)?
        .drain(..).collect();

        diesel::insert_into(schema::rel_station_schlagwort::table)
        .values(
            sw.iter()
            .map(|s| {
                (
                    schema::rel_station_schlagwort::stat_id.eq(stat_id),
                    schema::rel_station_schlagwort::sw_id.eq(idvec.get(s).unwrap())
                )}
            )
            .collect::<Vec<_>>()
        ).execute(connection)?;
    }

    return Ok(stat_id);
}
fn sanitize_string(s: &str) -> String{
    s.to_string()
}
pub fn insert_dokument(
    dok: models::Dokument,
    connection: &mut diesel::PgConnection) 
    -> Result<i32> {
    use schema::dokument::dsl;
    let did: i32 = diesel::insert_into(schema::dokument::table)
    .values(
        (
            dsl::titel.eq(sanitize_string(&dok.titel)),
            dsl::link.eq(dok.link),
            dsl::hash.eq(dok.hash),
            dsl::last_mod.eq(dok.last_mod.naive_utc()),
            dsl::zusammenfassung.eq(&dok.zusammenfassung.map(|s| sanitize_string(&s))),
            dsl::typ.eq(
                schema::dokumententyp::table.select(schema::dokumententyp::id)
                .filter(schema::dokumententyp::api_key.eq(&dok.typ.to_string()))
                .first::<i32>(connection)?
            ),
            dsl::volltext.eq(dok.volltext),
            dsl::drucksnr.eq(dok.drucksnr)
        )
    )
    .returning(dsl::id)
    .get_result::<i32>(connection)?;
    // Schlagworte
    if let Some(sw) = dok.schlagworte{
        diesel::insert_into(schema::schlagwort::table)
        .values(
            sw.iter()
            .map(|s|
                schema::schlagwort::api_key.eq(s))
            .collect::<Vec<_>>()
        )
        .on_conflict_do_nothing()
        .execute(connection)?;
        let idvec : HashMap<String, i32> = 
        schema::schlagwort::table
        .filter(schema::schlagwort::api_key.eq_any(&sw))
        .select((schema::schlagwort::api_key, schema::schlagwort::id))
        .get_results::<(String, i32)>(connection)?
        .drain(..).collect();

        diesel::insert_into(schema::rel_dok_schlagwort::table)
        .values(
            sw.iter()
            .map(|s| {
                (
                    schema::rel_dok_schlagwort::dok_id.eq(did),
                    schema::rel_dok_schlagwort::sw_id.eq(idvec.get(s).unwrap())
                )}
            )
            .collect::<Vec<_>>()
        ).execute(connection)?;
    }
    if let Some(auth) = dok.autoren{
        diesel::insert_into(schema::rel_dok_autor::table)
        .values(
            auth.iter()
            .map(|s|
                (
                    schema::rel_dok_autor::dok_id.eq(did),
                    schema::rel_dok_autor::autor.eq(s)
                )
            )
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }
    if let Some(autoren_personen) = dok.autorpersonen{
        diesel::insert_into(schema::rel_dok_autorperson::table)
        .values(
            autoren_personen.iter()
            .map(|s|
                (
                    schema::rel_dok_autorperson::dok_id.eq(did),
                    schema::rel_dok_autorperson::autor.eq(s)
                )
            )
            .collect::<Vec<_>>()
        )
        .execute(connection)?;
    }
    return Ok(did);
}
pub mod schema;
pub mod insert;
pub mod retrieve;
pub mod merge;

#[cfg(test)]
mod test{
    use diesel::Connection;
    use openapi::models;
    use crate::Result;

    #[test] 
    fn test_gsvh() -> Result<()>{
        let dok = models::Dokument{
            autoren: Some(vec!["Testautor".to_string()]),
            schlagworte: Some(vec!["Testschlagwort".to_string()]),
            hash: "Testhash".to_string(),
            titel: "Testtitel".to_string(),
            typ: models::Dokumententyp::Entwurf,
            link: "Testurl".to_string(),
            datum: chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            zusammenfassung: Some("Testzusammenfassung".to_string()),
        };

        let dok2 = models::Dokument{
            autoren: Some(vec!["Testautor".to_string()]),
            schlagworte: Some(vec!["Testschlagwort".to_string()]),
            hash: "Testhash".to_string(),
            titel: "Testtitel".to_string(),
            typ: models::Dokumententyp::Stellungnahme,
            link: "Testurl".to_string(),
            datum: chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            zusammenfassung: Some("Testzusammenfassung".to_string()),
        };
        let stellungnahme = models::Stellungnahme{
            dokument: dok2,
            lobbyregister_link: Some("URL".to_string()),
            meinung: Some(1)
        };
        let station = models::Station{
            dokumente: vec![dok],
            gremium: "Testgremium".to_string(),
            stellungnahmen: Some(vec![stellungnahme]),
            parlament: models::Parlament::By,
            trojaner: Some(true),
            link: Some("Testurl".to_string()),
            typ: models::Stationstyp::ParlAblehnung,
            schlagworte: Some(vec!["Testschlagwort".to_string()]),
            datum: chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        };
        let gsvh = models::Gesetzesvorhaben{
            api_id: uuid::Uuid::now_v7(),
            initiatoren: vec!["Testinitiator".to_string()],
            links: Some(vec!["Testlink".to_string()]),
            titel: "Testtitel".to_string(),
            typ: models::Gesetzestyp::BggEinspruch,
            verfassungsaendernd: false,
            ids: Some(vec![models::Identifikator{id: "Testid".to_string(), typ: models::Identifikatortyp::Initdrucks}]),
            stationen: vec![station]
        };
        let mut conn = diesel::pg::PgConnection::establish(
            std::env::var("DATABASE_URL")?.as_str()
        ).unwrap();
        
        let _ = conn.test_transaction( move | mut conn| 
            {
                let id = super::insert::insert_gsvh(&gsvh, &mut conn);

                id
            }
        );
        Ok(())
    }
}
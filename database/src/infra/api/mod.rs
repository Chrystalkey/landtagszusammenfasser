use crate::error::DatabaseError;
use chrono::{DateTime, Utc};
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::async_db;

pub struct Response {
    pub payload: Vec<Gesetzesvorhaben>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum Gesetzestyp {
    Einspruchsgesetz,
    Zustimmungsgesetz,
    Volksbegehren,
    Sonstig,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum IdentifikatorTyp {
    Drucksachennummer,
    Vorgangsnummer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Identifikator {
    id: String,
    typ: IdentifikatorTyp,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Gesetzesvorhaben {
    /// The unique identifier of the Gesetzesvorhaben within our cosmos
    api_id: Uuid,
    /// The title of it
    titel: String,
    /// If it requires changing the constitution
    verfassungsaendernd: bool,
    /// if there is a trojaner suspected
    trojaner: bool,
    /// who initiated it
    initiative: String,
    /// the type of it
    typ: Gesetzestyp,
    /// other ids
    ids: Vec<Identifikator>,
    /// associated links
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    links: Vec<String>,
    /// associated Notes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    notes: Vec<String>,
    /// Stationen it has passed
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    stationen: Vec<Station>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum Stationstyp {
    EntwurfReferentenentwurf,
    EntwurfEckpunktepapier,
    ParlamentInitiative,
    ParlamentKabinettsbeschluss,
    ParlamentBeschlussempfehlung,
    ParlamentPlenarsitzung,
    ParlamentBeschluss,
    Inkraftgetreten,
    Abgelehnt,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum Parlament {
    /// Bundestag
    BT,
    /// Bundesrat
    BR,
    /// Bundesversammlung
    BV,
    /// Europakammer
    EK,
    /// Brandenburger Landtag
    BB,
    /// Bayerischer Landtag
    BY,
    /// Berliner Landtag
    BE,
    /// Bremischer Landtag
    HB,
    /// Hamburgischer Landtag
    HH,
    /// Hessischer Landtag
    HE,
    /// Mecklenburg-Vorpommerscher Landtag
    MV,
    /// Niedersächsischer Landtag
    NI,
    /// Nordrhein-Westfälischer Landtag
    NW,
    /// Rheinland-Pfälzischer Landtag
    RP,
    /// Saarländischer Landtag
    SL,
    /// Sächsischer Landtag
    SN,
    /// Thüringer Landtag
    TH,
    /// Schleswig-Holsteinischer Landtag
    SH,
    /// Baden-Württembergischer Landtag
    BW,
    /// Sachsen-Anhaltischer Landtag
    ST,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Station {
    pub stationstyp: Stationstyp,
    pub datum: DateTime<Utc>,
    pub url: Option<String>,
    pub parlament: Parlament,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub schlagworte: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub dokumente: Vec<Dokument>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub stellungnahmen: Vec<Stellungnahme>,
    /// Entweder Plenum oder der Name eines Ausschusses
    pub zuordnung: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Stellungnahme {
    pub datum: DateTime<Utc>,
    pub titel: String,
    pub dokument: Dokument,
    pub meinung: i32,
    pub url: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum DokumentTyp {
    /// Was man vom Ministerium bekommt
    Gesetzesentwurf,
    /// Was Ausschüsse abgeben oder als Kabinettsbeschluss rausgeht
    Beschlussempfehlung,
    /// Was das Parlament beschließt
    Beschluss,
    /// Was die Zivilgesellschaft abgibt
    Stellungnahme,
    /// Parlamentsprotokolle
    Protokoll,
    /// Catch-All
    Sonstiges,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Autor {
    pub name: String,
    pub organisation: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Dokument {
    pub titel: String,
    pub url: String,
    pub hash: String,
    pub zusammenfassung: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub schlagworte: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub autoren: Vec<Autor>,
    pub typ: DokumentTyp,
}

macro_rules! match_enum {
    ($enum:ident, $value:ident, $($variant:ident),+) => {
        match $value.as_str(){
            $(
                stringify!($variant) => Ok($enum::$variant),
            )+
            _ => Err(format!("Invalid Enum Value for enum {}: {}", stringify!($enum), stringify!($value)).into()),
        }
    };
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for Gesetzestyp
where
    DB: diesel::backend::Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        match_enum!(
            Gesetzestyp,
            s,
            Einspruchsgesetz,
            Zustimmungsgesetz,
            Volksbegehren,
            Sonstig
        )
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for Stationstyp
where
    DB: diesel::backend::Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        match_enum!(
            Stationstyp,
            s,
            EntwurfReferentenentwurf,
            EntwurfEckpunktepapier,
            ParlamentInitiative,
            ParlamentKabinettsbeschluss,
            ParlamentBeschlussempfehlung,
            ParlamentPlenarsitzung,
            ParlamentBeschluss,
            Inkraftgetreten,
            Abgelehnt
        )
    }
}
impl<DB> FromSql<diesel::sql_types::Text, DB> for DokumentTyp
where
    DB: diesel::backend::Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        match_enum!(
            DokumentTyp,
            s,
            Gesetzesentwurf,
            Beschlussempfehlung,
            Beschluss,
            Stellungnahme,
            Protokoll,
            Sonstiges
        )
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for Parlament
where
    DB: diesel::backend::Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        match_enum!(
            Parlament, s, BT, BR, BV, EK, BB, BY, BE, HB, HH, HE, MV, NI, NW, RP, SL, SN, TH, SH,
            BW, ST
        )
    }
}
impl Gesetzesvorhaben {
    pub async fn construct_from(
        object: crate::infra::db::connection::Gesetzesvorhaben,
        conn: &mut deadpool_diesel::postgres::Connection,
    ) -> Result<Self, crate::error::DatabaseError> {
        use crate::infra::db::schema as dbs;
        use diesel::prelude::*;

        let db_stationen: Vec<crate::infra::db::connection::Station> = async_db!(conn, load, {
            dbs::station::table.filter(dbs::station::gesvh_id.eq(object.id))
        });
        let mut stations = vec![];
        for station in db_stationen {
            let station = Station::construct_from(station, conn).await?;
            stations.push(station);
        }
        let mut ids: Vec<(String, String)> = async_db!(conn, load, {
            dbs::rel_gesvh_id::table
                .inner_join(dbs::identifikatortyp::table)
                .filter(dbs::rel_gesvh_id::gesetzesvorhaben_id.eq(object.id))
                .select((
                    dbs::rel_gesvh_id::identifikator,
                    dbs::identifikatortyp::value,
                ))
        });
        let ids = ids
            .drain(..)
            .map(|(id, typ)| Identifikator {
                id,
                typ: match typ.as_str() {
                    "drucksachennummer" => IdentifikatorTyp::Drucksachennummer,
                    "vorgangsnummer" => IdentifikatorTyp::Vorgangsnummer,
                    _ => unimplemented!(),
                },
            })
            .collect();

        return Ok(Self {
            api_id: object.api_id,
            titel: object.titel,
            verfassungsaendernd: object.verfassungsaendernd,
            trojaner: object.trojaner,
            initiative: object.initiative,
            typ: async_db!(conn, first, {
                dbs::gesetzestyp::table
                    .filter(dbs::gesetzestyp::id.eq(object.typ))
                    .select(dbs::gesetzestyp::value)
            }),
            ids,
            links: async_db!(conn, load, {
                dbs::rel_gesvh_links::table
                    .filter(dbs::rel_gesvh_links::gesetzesvorhaben_id.eq(object.id))
                    .select(dbs::rel_gesvh_links::link)
            }),
            notes: async_db!(conn, load, {
                dbs::rel_gesvh_notes::table
                    .filter(dbs::rel_gesvh_notes::gesetzesvorhaben_id.eq(object.id))
                    .select(dbs::rel_gesvh_notes::note)
            }),
            stationen: stations,
        });
    }
}
impl Station {
    pub async fn construct_from(
        object: crate::infra::db::connection::Station,
        conn: &mut deadpool_diesel::postgres::Connection,
    ) -> Result<Self, crate::error::DatabaseError> {
        use crate::infra::db::schema as dbs;
        use diesel::prelude::*;
        let db_doks: Vec<crate::infra::db::connection::Dokument> = async_db!(conn, load, {
            dbs::dokument::table
                .inner_join(dbs::rel_station_dokument::table)
                .filter(dbs::rel_station_dokument::station_id.eq(object.id))
                .select(dbs::dokument::all_columns)
        });
        let mut dokumente = vec![];
        for dok in db_doks {
            dokumente.push(Dokument::construct_from(dok, conn).await?);
        }
        let db_stellungnahmen: Vec<crate::infra::db::connection::Stellungnahme> =
            async_db!(conn, load, {
                dbs::stellungnahme::table
                    .filter(dbs::stellungnahme::station_id.eq(object.id))
                    .select(dbs::stellungnahme::all_columns)
            });
        let mut stellungnahmen = vec![];
        for stellungnahme in db_stellungnahmen {
            stellungnahmen.push(Stellungnahme::construct_from(stellungnahme, conn).await?);
        }
        return Ok(Self {
            stationstyp: async_db!(conn, first, {
                dbs::stationstyp::table
                    .filter(dbs::stationstyp::id.eq(object.stationstyp))
                    .select(dbs::stationstyp::value)
            }),
            datum: object.zeitpunkt.and_utc(),
            url: object.url,
            parlament: async_db!(conn, first, {
                dbs::parlament::table
                    .filter(dbs::parlament::id.eq(object.parlament))
                    .select(dbs::parlament::value)
            }),
            schlagworte: async_db!(conn, load, {
                dbs::rel_station_schlagwort::table
                    .filter(dbs::rel_station_schlagwort::station_id.eq(object.id))
                    .inner_join(dbs::schlagwort::table)
                    .select(dbs::schlagwort::value)
            }),
            dokumente,
            stellungnahmen,
            zuordnung: object.zuordnung,
        });
    }
}
impl Stellungnahme {
    pub async fn construct_from(
        object: crate::infra::db::connection::Stellungnahme,
        conn: &mut deadpool_diesel::postgres::Connection,
    ) -> Result<Self, crate::error::DatabaseError> {
        use diesel::prelude::*;
        return Ok(Self {
            datum: object.zeitpunkt.and_utc(),
            titel: object.titel,
            dokument: Dokument::construct_from(
                async_db!(conn, first, {
                    crate::infra::db::schema::dokument::table
                        .filter(crate::infra::db::schema::dokument::id.eq(object.dokument_id))
                }),
                conn,
            )
            .await?,
            meinung: object.meinung,
            url: object.url,
        });
    }
}

impl Dokument {
    pub async fn construct_from(
        object: crate::infra::db::connection::Dokument,
        conn: &mut deadpool_diesel::postgres::Connection,
    ) -> Result<Self, crate::error::DatabaseError> {
        use diesel::prelude::*;
        let mut autoren_db: Vec<(String, String)> = async_db!(conn, load, {
            crate::infra::db::schema::rel_dok_autor::table
                .filter(crate::infra::db::schema::rel_dok_autor::dokument_id.eq(object.id))
                .inner_join(crate::infra::db::schema::autor::table)
                .select((
                    crate::infra::db::schema::autor::name,
                    crate::infra::db::schema::autor::organisation,
                ))
        });
        let autoren = autoren_db
            .drain(..)
            .map(|(name, org)| Autor {
                name,
                organisation: org,
            })
            .collect();

        return Ok(Self {
            titel: object.titel,
            hash: object.hash,
            url: object.url,
            zusammenfassung: object.zusammenfassung,
            schlagworte: async_db!(conn, load, {
                crate::infra::db::schema::rel_dok_schlagwort::table
                    .filter(crate::infra::db::schema::rel_dok_schlagwort::dokument_id.eq(object.id))
                    .inner_join(crate::infra::db::schema::schlagwort::table)
                    .select(crate::infra::db::schema::schlagwort::value)
            }),
            autoren,
            typ: async_db!(conn, first, {
                crate::infra::db::schema::dokumenttyp::table
                    .filter(crate::infra::db::schema::dokumenttyp::id.eq(object.dokumenttyp_id))
                    .select(crate::infra::db::schema::dokumenttyp::value)
            }),
        });
    }
}

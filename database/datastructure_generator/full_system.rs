struct ParlamenteDB{
	pub id: i32,
	pub name: String,
	pub kurzname: String,
}
struct AbstimmungsergebnisseDB{
	pub id: i32,
	pub abstimmung: Option<i32>,
	pub fraktion: Option<i32>,
	pub anteil: f32,
}
struct Schlagworte{
	pub id: i32,
	pub schlagwort: String,
	pub beschreibung: String,
}
struct Dokumenttypen{
	pub id: i32,
	pub name: String,
}
struct Abstimmungsergebnisse{
	pub id: Option<i32>,
	pub abstimmung: i32,
	pub fraktion: i32,
	pub anteil: f32,
}
struct RelGesSchlagworteDB{
	pub gesetzesvorhaben: Option<uuid::Uuid>,
	pub schlagwort: Option<i32>,
}
struct SonstigeIds{
	pub id: i32,
	pub typ: String,
	pub inhalt: String,
}
struct Ausschussberatungen{
	pub id: i32,
	pub datum: chrono::DateTime<chrono::Utc>,
	pub ausschuss: Option<i32>,
	pub dokument: Option<i32>,
}
struct Initiatoren{
	pub id: i32,
	pub name: String,
	pub organisation: String,
	pub url: String,
}
struct StatusDB{
	pub id: i32,
	pub name: String,
	pub parlament: Option<i32>,
}
struct InitiatorenDB{
	pub id: i32,
	pub name: String,
	pub organisation: String,
	pub url: String,
}
struct Fraktionen{
	pub id: i32,
	pub name: String,
}
struct DokumenttypenDB{
	pub id: i32,
	pub name: String,
}
struct Status{
	pub id: i32,
	pub name: String,
}
struct GesetzesvorhabenDB{
	pub id: uuid::Uuid,
	pub titel: String,
	pub off_titel: String,
	pub url_gesblatt: Option<String>,
	pub id_gesblatt: Option<String>,
	pub verfassungsaendernd: bool,
	pub trojaner: Option<bool>,
	pub federfuehrung: Option<i32>,
	pub initiator: Option<i32>,
}
struct Dokumente{
	pub id: i32,
	pub off_id: String,
	pub datum: chrono::DateTime<chrono::Utc>,
	pub url: String,
	pub collector_url: String,
	pub file: Option<String>,
	pub hash: String,
	pub typ: Option<i32>,
}
struct RelGesTopsDB{
	pub top: Option<i32>,
	pub gesetzesvorhaben: Option<uuid::Uuid>,
	pub abstimmung: Option<i32>,
	pub dokument: Option<i32>,
	pub titel: String,
}
struct SonstigeIdsDB{
	pub id: i32,
	pub gesetzesvorhaben: Option<uuid::Uuid>,
	pub typ: String,
	pub inhalt: String,
}
struct Tops{
	pub id: i32,
	pub datum: chrono::DateTime<chrono::Utc>,
	pub url: String,
	pub tagesordnungspunkt: Vec<Tagesordnungspunkt>,
}
struct RelGesSchlagworte{
	pub gesetzesvorhaben: uuid::Uuid,
	pub schlagwort: i32,
}
struct RelGesStatus{
	pub gesetzesvorhaben: uuid::Uuid,
	pub status: i32,
	pub abstimmung: i32,
	pub datum: chrono::DateTime<chrono::Utc>,
}
struct Tagesordnungspunkt{
	pub id: i32,
	pub titel: String,
	pub document: Option<i32>,
	pub abstimmung: Option<i32>,
}
struct GesetzeseigenschaftenDB{
	pub id: i32,
	pub eigenschaft: String,
}
struct Abstimmungstyp{
	pub id: i32,
	pub name: String,
}
struct AbstimmungenDB{
	pub id: i32,
	pub namentlich: bool,
	pub url: String,
	pub typ: Option<i32>,
	pub gesetzesvorhaben: Option<uuid::Uuid>,
}
struct Parlamente{
	pub id: i32,
	pub name: String,
	pub kurzname: String,
	pub ausschuesse: Vec<Ausschuesse>,
	pub tops: Vec<Tops>,
	pub status: Vec<Status>,
}
struct Ausschuesse{
	pub id: i32,
	pub name: String,
}
struct TagesordnungspunktDB{
	pub id: i32,
	pub titel: String,
	pub tops_id: Option<i32>,
	pub document: Option<i32>,
	pub abstimmung: Option<i32>,
}
struct RelGesEigenschaftDB{
	pub gesetzesvorhaben: Option<uuid::Uuid>,
	pub eigenschaft: Option<i32>,
}
struct SchlagworteDB{
	pub id: i32,
	pub schlagwort: String,
	pub beschreibung: String,
}
struct TopsDB{
	pub id: i32,
	pub datum: chrono::DateTime<chrono::Utc>,
	pub url: String,
	pub parlament: Option<i32>,
}
struct RelGesEigenschaft{
	pub gesetzesvorhaben: uuid::Uuid,
	pub eigenschaft: i32,
}
struct AbstimmungstypDB{
	pub id: i32,
	pub name: String,
}
struct AusschussberatungenDB{
	pub id: i32,
	pub datum: chrono::DateTime<chrono::Utc>,
	pub gesetzesvorhaben: Option<uuid::Uuid>,
	pub ausschuss: Option<i32>,
	pub dokument: Option<i32>,
}
struct RelGesStatusDB{
	pub gesetzesvorhaben: Option<uuid::Uuid>,
	pub status: Option<i32>,
	pub abstimmung: Option<i32>,
	pub datum: chrono::DateTime<chrono::Utc>,
}
struct Abstimmungen{
	pub id: i32,
	pub namentlich: bool,
	pub url: String,
	pub typ: Option<i32>,
}
struct DokumenteDB{
	pub id: i32,
	pub off_id: String,
	pub datum: chrono::DateTime<chrono::Utc>,
	pub url: String,
	pub collector_url: String,
	pub file: Option<String>,
	pub hash: String,
	pub gesetzesvorhaben: Option<uuid::Uuid>,
	pub typ: Option<i32>,
}
struct RelGesTops{
	pub top: Option<i32>,
	pub gesetzesvorhaben: uuid::Uuid,
	pub abstimmung: i32,
	pub dokument: i32,
	pub titel: String,
}
struct Gesetzeseigenschaften{
	pub id: i32,
	pub eigenschaft: String,
}
struct FraktionenDB{
	pub id: i32,
	pub name: String,
}
struct Gesetzesvorhaben{
	pub id: uuid::Uuid,
	pub titel: String,
	pub off_titel: String,
	pub url_gesblatt: Option<String>,
	pub id_gesblatt: Option<String>,
	pub verfassungsaendernd: bool,
	pub trojaner: Option<bool>,
	pub federfuehrung: Option<i32>,
	pub initiator: Option<i32>,
	pub dokumente: Vec<Dokumente>,
	pub ausschussberatungen: Vec<Ausschussberatungen>,
	pub sonstige_ids: Vec<SonstigeIds>,
	pub abstimmungen: Vec<Abstimmungen>,
}
struct AusschuesseDB{
	pub id: i32,
	pub name: String,
	pub parlament: Option<i32>,
}

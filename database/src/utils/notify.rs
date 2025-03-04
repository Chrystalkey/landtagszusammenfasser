
use lettre::{Message, Transport};
use crate::{LTZFServer, Result};
use uuid::Uuid;

impl LTZFServer{
    /// guarded to String conversion
    pub fn guard_ts<T: ToString>(&self, input: T, api_id : Uuid, object: &str) -> Result<String>{
        let temp = input.to_string();
        if temp == "sonstig"{
            notify_unknown_variant::<T>(api_id, object, self)?
        }
        return Ok(temp);
    }
}

pub fn notify_new_enum_entry<T: std::fmt::Debug>(
    identifier: Uuid,
    object: &str,
    new_entry: &T,
    server: &LTZFServer,
) -> Result<()> {
    let subject = format!("Neuer Eintrag des Typs `{}` wurde generiert, da nicht vorhanden. Auf Konsistenz mit dem Datensatz überprüfen.",stringify!(T));
    let body = format!(
        "Während einer Insert Operation für {} `{}` wurde der Eintrag `{:?}` neu generiert.\n
        ", object, identifier, new_entry
    );
    send_email(subject, body, server)?;

    Ok(())
}
pub fn notify_ambiguous_match<T: std::fmt::Debug>(
    api_ids: Vec<Uuid>,
    object: &T,
    at_loc: &str,
    server: &LTZFServer
) -> Result<()> {
    let subject = format!("Mehrere Datensätze gefunden die neuem Objekt ähnlich sind. Bitte um Konfliktauflösung.");
    let body = format!(
        "Während: `{}` wurde folgendes Objekt wurde hochgeladen: {:#?}.
        Folgende Objekte in der Datenbank sind ähnlich: {:#?}", at_loc, object, api_ids
    );
    send_email(subject, body, server)?;
    Ok(())
}

pub fn notify_unknown_variant<T>(
    api_id: Uuid,
    object: &str,
    server: &LTZFServer,
)->Result<()>{
    let topic = format!("Für {} `{}` wurde `sonstig` angegeben als Wert für `{}`",
    object, api_id, stringify!(T));
    send_email(topic, String::new(), server)?;

    todo!("Notify the admin when a 'sonstig' enum variant is unwrapped")
}

pub fn send_email(subject: String, body: String, state: &LTZFServer) -> Result<()> {
    if state.mailer.is_none() {
        return Ok(());
    }
    let email = Message::builder()
        .from(
            format!("Landtagszusammenfasser <{}>", state.config.mail_sender.as_ref().unwrap())
                .parse()
                .unwrap(),
        )
        .to(state.config.mail_recipient.as_ref().unwrap().parse().unwrap())
        .subject(subject.clone())
        .body(body.clone())
        .unwrap();
    tracing::info!("Mail was Sent. Subject: {}", subject);
    tracing::debug!("Mail Contents:\n{}", body);
    state.mailer.as_ref().unwrap().send(&email)?;
    Ok(())
}
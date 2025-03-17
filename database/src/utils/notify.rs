use crate::{error::DataValidationError, LTZFServer, Result};
use lettre::{message::header::ContentType, Message, Transport};
use uuid::Uuid;

impl LTZFServer {
    /// guarded to String conversion
    pub fn guard_ts<T: ToString>(&self, input: T, api_id: Uuid, object: &str) -> Result<String> {
        let temp = input.to_string();
        if temp == "sonstig" {
            notify_unknown_variant::<T>(api_id, object, self)?
        }
        return Ok(temp);
    }
}

pub fn notify_new_enum_entry<T: std::fmt::Debug + ToString>(
    new_entry: &T,
    similarity: Vec<(f32, T)>,
    server: &LTZFServer,
) -> Result<()> {
    let subject = format!(
        "Für Typ `{}` wurde ein neuer Eintrag `{:?}` erstellt. ",
        std::any::type_name::<T>(),
        new_entry
    );

    let simstr = similarity
        .iter()
        .map(|(p, t)| format!("{}: {}", p.to_string(), t.to_string()))
        .fold("".to_string(), |a, n| format!("{a}\n{n}"));

    let body = format!("Es gibt {} ähnliche Einträge: {simstr}", similarity.len());
    send_email(subject.clone(), body.clone(), server)?;
    tracing::error!("Notify: New Enum Entry: {}\n{}!", subject, body);

    Ok(())
}
pub fn notify_ambiguous_match<T: std::fmt::Debug + serde::Serialize>(
    api_ids: Vec<Uuid>,
    object: &T,
    during_operation: &str,
    server: &LTZFServer,
) -> Result<()> {
    let subject = format!(
        "Ambiguous Match: Während {}", during_operation
    );
    let body = format!(
        "Während: `{}` wurde folgendes Objekt wurde hochgeladen: {}.
        Folgende Objekte in der Datenbank sind ähnlich: {:#?}",
        during_operation, 
        serde_json::to_string_pretty(object)
        .map_err(|e| DataValidationError::InvalidFormat { field: "passed obj for ambiguous match".to_string(), message: e.to_string() })?, api_ids
    );
    send_email(subject, body, server)?;
    tracing::error!("Notify: Ambiguous Match! Sending mails is not yet supported.");
    Ok(())
}

pub fn notify_unknown_variant<T>(api_id: Uuid, object: &str, server: &LTZFServer) -> Result<()> {
    let topic = format!(
        "Für {} `{}` wurde `sonstig` angegeben als Wert für `{}`",
        object,
        api_id,
        std::any::type_name::<T>()
    );
    send_email(topic, String::new(), server)?;
    tracing::error!("Notify: Unknown Variant! Sending mails is not yet supported.");
    Ok(())
}

pub fn send_email(subject: String, body: String, state: &LTZFServer) -> Result<()> {
    if state.mailer.is_none() {
        return Ok(());
    }
    let email = Message::builder()
        .from(
            format!(
                "Landtagszusammenfasser <{}>",
                state.config.mail_sender.as_ref().unwrap()
            )
            .parse()
            .map_err(|e| DataValidationError::InvalidFormat { field: "mail address".to_string(), message: format!("{}", e) })?,
        )
        .to(state
            .config
            .mail_recipient
            .as_ref()
            .unwrap()
            .parse()
            .map_err(|e| DataValidationError::InvalidFormat { field: "mail address".to_string(), message: format!("{}", e) })?)
        .subject(subject.clone())
        .header(ContentType::TEXT_PLAIN)
        .body(body.clone())
        .unwrap();
    tracing::info!("Mail was Sent. Subject: {}", subject);
    tracing::debug!("Mail Contents:\n{}", body);
    state.mailer.as_ref().unwrap().send(&email)?;
    Ok(())
}

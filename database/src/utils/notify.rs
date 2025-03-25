use std::{sync::{Arc, RwLock}, time::Duration};

use crate::{error::DataValidationError, LTZFServer, Result};
use lettre::{message::header::ContentType, Message, Transport};
use uuid::Uuid;

enum MailNotificationType{
    EnumAdded,
    SonstigUnwrapped,
    AmbiguousMatch,
    Other,
}
struct Mail {
    subject: String,
    body: String,
    tp: MailNotificationType
}

pub struct MailBundle{
    mailthread: Option<std::thread::JoinHandle<()>>,
    kill: Arc<RwLock<bool>>,
    cache: Arc<RwLock<Vec<Mail>>>,
}
impl MailBundle {
    pub async fn new(
        config: &crate::Configuration
    ) ->Result<Option<Self>>{
        let cm= config.build_mailer().await;
        if let Err(e) = cm{
            tracing::warn!(
                "Failed to create mailer: {}\nMailer will not be available",
                e
            );
            return Ok(None);
        }
        let kill = Arc::new(RwLock::new(false));
        let kclone = kill.clone();

        let cache: Arc<RwLock<Vec<Mail>>> = Arc::new(RwLock::new(vec![]));
        let cclone = cache.clone();
        let sender : lettre::message::Mailbox= format!(
            "Landtagszusammenfasser <{}>",
            config.mail_sender.as_ref().unwrap(),
        )
        .parse()
        .map_err(|e| DataValidationError::InvalidFormat { field: "mail address".to_string(), message: format!("{}", e) })?;
        let recipient : lettre::message::Mailbox = config
            .mail_recipient
            .as_ref()
            .unwrap()
            .parse()
            .map_err(|e| DataValidationError::InvalidFormat { field: "mail address".to_string(), message: format!("{}", e) })?;

        let thread = std::thread::spawn(
            move || {
                let mref = kclone;
                let mailer = cm.unwrap();
                let sender = sender;
                let recipient = recipient;
                while !*mref.read().unwrap() {
                    while !*mref.read().unwrap(){
                        std::thread::sleep(Duration::from_secs(1));
                    }
                    if cclone.read().unwrap().is_empty() {
                        continue;
                    }
                    let mut ambiguous_match = vec![];
                    let mut variant_added = vec![];
                    let mut sonstig_unwrapped = vec![];
                    let mut other = vec![]; 

                    for mail in cclone.write().unwrap().drain(..){
                        match mail.tp {
                            MailNotificationType::AmbiguousMatch=>ambiguous_match.push(mail),
                            MailNotificationType::EnumAdded=>variant_added.push(mail),
                            MailNotificationType::SonstigUnwrapped=>sonstig_unwrapped.push(mail),
                            MailNotificationType::Other => other.push(mail)
                        }
                    }
                    let (s_am, s_va, s_su, s_ot) = (ambiguous_match.len(), variant_added.len(), sonstig_unwrapped.len(), other.len());

                    if s_am != 0{
                        let ambiguous_match_body = ambiguous_match.iter().fold("".to_string(), |a,n|{
                            format!("{a}\n======================={}\n\n{}", n.subject, n.body)
                        });
                        let email = Message::builder()
                            .from(sender.clone())
                            .to(recipient.clone())
                            .subject(format!("Found {} ambiguous matches since last check", s_am))
                            .header(ContentType::TEXT_PLAIN)
                            .body(ambiguous_match_body)
                            .unwrap();
                        mailer.send(&email).unwrap();
                    }
                    if s_va != 0{
                        let variant_added_body = variant_added.iter().fold("".to_string(), |a,n|{
                            format!("{a}\n======================={}\n\n{}", n.subject, n.body)
                        });
                        let email = Message::builder()
                            .from(sender.clone())
                            .to(recipient.clone())
                            .subject(format!("Added {} new variants since last check", s_va))
                            .header(ContentType::TEXT_PLAIN)
                            .body(variant_added_body)
                            .unwrap();
                        mailer.send(&email).unwrap();
                    }
                    if s_su != 0{
                        let sonstig_unwrapped_body = sonstig_unwrapped.iter().fold("".to_string(), |a,n|{
                            format!("{a}\n======================={}\n\n{}", n.subject, n.body)
                        });
                        let email = Message::builder()
                            .from(sender.clone())
                            .to(recipient.clone())
                            .subject(format!("{} sonstig's unwrapped since last check", s_su))
                            .header(ContentType::TEXT_PLAIN)
                            .body(sonstig_unwrapped_body)
                            .unwrap();
                        mailer.send(&email).unwrap();
                    }
                    if s_ot != 0{
                        let other_body = other.iter().fold("".to_string(), |a,n|{
                            format!("{a}\n======================={}\n\n{}", n.subject, n.body)
                        });
                        let email = Message::builder()
                            .from(sender.clone())
                            .to(recipient.clone())
                            .subject(format!("{} Other messages since last check", s_ot))
                            .header(ContentType::TEXT_PLAIN)
                            .body(other_body)
                            .unwrap();
                        mailer.send(&email).unwrap();
                    }
                }
            }
        );
        Ok(Some(Self{
            cache,
            mailthread: Some(thread),
            kill,
        }))
    }
    async fn send(&self, mail: Mail) -> Result<()>{
        self.cache.write().unwrap().push(mail);
        Ok(())
    }
}

impl Drop for MailBundle{
    fn drop(&mut self) {
        *self.kill.write().unwrap() = false;
        if let Some(handle) = self.mailthread.take(){
            handle.join().unwrap()
        }
    }
}

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
    if server.mailbundle.is_none(){
        return Ok(());
    }
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
    tracing::warn!("Notify: New Enum Entry: {}\n{}!", subject, body);
    server.mailbundle.as_ref().unwrap().send(Mail { subject, body, tp: MailNotificationType::EnumAdded });

    Ok(())
}
pub fn notify_ambiguous_match<T: std::fmt::Debug + serde::Serialize>(
    api_ids: Vec<Uuid>,
    object: &T,
    during_operation: &str,
    server: &LTZFServer,
) -> Result<()> {
    if server.mailbundle.is_none(){
        return Ok(());
    }
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
    tracing::error!("Notify: Ambiguous Match!");
    server.mailbundle.as_ref().unwrap().send(Mail { subject, body, tp: MailNotificationType::AmbiguousMatch });
    Ok(())
}

pub fn notify_unknown_variant<T>(api_id: Uuid, object: &str, server: &LTZFServer) -> Result<()> {
    if server.mailbundle.is_none(){
        return Ok(());
    }
    let subject = format!(
        "Für {} `{}` wurde `sonstig` angegeben als Wert für `{}`",
        object,
        api_id,
        std::any::type_name::<T>()
    );
    tracing::error!("Notify: Unknown Variant! Sending mails is not yet supported.");
    server.mailbundle.as_ref().unwrap().send(Mail { subject, body: "".to_string(), tp: MailNotificationType::SonstigUnwrapped });
    Ok(())
}

use std::sync::Arc;

use lettre::{Message, Transport};

use crate::AppState;

pub fn send_email(subject: String, body: String, state: Arc<AppState>) {
    let email = Message::builder()
        .from(
            format!("Landtagszusammenfasser <{}>", state.config.mail_sender)
                .parse()
                .unwrap(),
        )
        .to(state.config.mail_recipient.parse().unwrap())
        .subject(subject.clone())
        .body(body.clone())
        .unwrap();
    tracing::info!("Mail was Sent. Subject: {}", subject);
    tracing::debug!("Mail Contents:\n{}", body);
    state.mailer.send(&email).unwrap();
}

pub fn no_match_found(message: String, state: Arc<AppState>){
    send_email(
        "No Match Found: Human intervention necessary".to_string(),
        message,
        state
    );
}

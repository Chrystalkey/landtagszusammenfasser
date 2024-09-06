use std::sync::Arc;

use lettre::{Message, Transport};

use crate::AppState;

pub async fn send_email(subject: String, body: String, state: Arc<AppState>) {
    let email = Message::builder()
        .from(
            format!("Landtagszusammenfasser <{}>", state.config.mail_sender)
                .parse()
                .unwrap(),
        )
        .to(state.config.mail_recipient.parse().unwrap())
        .subject(subject)
        .body(body)
        .unwrap();
    state.mailer.send(&email).unwrap();
}

pub async fn no_match_found(message: String, state: Arc<AppState>){
    send_email(
        "No Match Found: Human intervention necessary".to_string(),
        message,
        state
    ).await;
}

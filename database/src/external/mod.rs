use std::sync::Arc;

use lettre::{Message, Transport};

use crate::AppState;

async fn send_email(subject: String, body: String, state: Arc<AppState>) {
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

async fn data_inconsistency(
    message: String,
    state: Arc<AppState>
){
    send_email(
        "Data Inconsistency: Human intervention necessary".to_string(),
        message,
        state
    ).await;
}

async fn database_error(
    message: String,
    state: Arc<AppState>
){
    send_email(
        "Database Error: Human intervention necessary".to_string(),
        message,
        state
    ).await;
}

async fn internal_error(
    message: String,
    state: Arc<AppState>
){
    send_email(
        "Internal Server Error: Human intervention necessary".to_string(),
        message,
        state
    ).await;
}
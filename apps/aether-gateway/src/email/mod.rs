mod delivery;
mod smtp;
mod templates;
mod worker;

pub(crate) use delivery::{execute_email_delivery, queue_email_delivery, EmailDeliveryPayload};
pub(crate) use smtp::{
    send_email_blocking, test_smtp_connection_blocking, EmailMessage, SmtpConfig, SmtpTestResult,
};
pub(crate) use templates::{
    build_password_reset_email_payload, build_test_email_payload, build_verification_email_payload,
};
pub(crate) use worker::spawn_auth_email_delivery_worker;

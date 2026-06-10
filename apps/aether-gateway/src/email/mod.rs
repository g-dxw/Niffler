mod delivery;
mod smtp;
mod templates;
mod worker;

pub(crate) use delivery::{EmailDeliveryPayload, execute_email_delivery, queue_email_delivery};
pub(crate) use smtp::{
    EmailMessage, SmtpConfig, SmtpTestResult, send_email_blocking, test_smtp_connection_blocking,
};
pub(crate) use templates::{build_test_email_payload, build_verification_email_payload};
pub(crate) use worker::spawn_auth_email_delivery_worker;

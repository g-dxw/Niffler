mod smtp;

pub(crate) use smtp::{
    EmailMessage, SmtpConfig, SmtpTestResult, send_email_blocking, test_smtp_connection_blocking,
};

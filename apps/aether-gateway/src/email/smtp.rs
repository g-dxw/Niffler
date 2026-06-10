use crate::GatewayError;
use base64::Engine;
use std::io::{BufRead, Write};
use std::time::Duration;

const SMTP_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub(crate) struct SmtpConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) use_tls: bool,
    pub(crate) use_ssl: bool,
    pub(crate) from_email: String,
    pub(crate) from_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct EmailMessage {
    pub(crate) to_email: String,
    pub(crate) subject: String,
    pub(crate) html_body: String,
    pub(crate) text_body: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct SmtpTestResult {
    pub(crate) success: bool,
    pub(crate) message: String,
}

fn encode_mime_header(value: &str) -> String {
    if value.is_ascii() {
        return value.to_string();
    }
    format!(
        "=?UTF-8?B?{}?=",
        base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
    )
}

fn wrap_base64(value: &str) -> String {
    let mut wrapped = String::new();
    for chunk in value.as_bytes().chunks(76) {
        wrapped.push_str(std::str::from_utf8(chunk).unwrap_or_default());
        wrapped.push_str("\r\n");
    }
    wrapped
}

fn build_tls_config() -> std::sync::Arc<rustls::ClientConfig> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    std::sync::Arc::new(config)
}

fn resolve_server_name(host: &str) -> Result<rustls::pki_types::ServerName<'static>, GatewayError> {
    let host = host.trim().trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return Ok(rustls::pki_types::ServerName::from(ip));
    }
    rustls::pki_types::ServerName::try_from(host.to_string())
        .map_err(|err| GatewayError::Internal(err.to_string()))
}

fn connect_tcp_stream(config: &SmtpConfig) -> Result<std::net::TcpStream, GatewayError> {
    let stream = std::net::TcpStream::connect((config.host.as_str(), config.port))
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(SMTP_TIMEOUT_SECS)))
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(SMTP_TIMEOUT_SECS)))
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    Ok(stream)
}

fn wrap_tls_stream(
    stream: std::net::TcpStream,
    host: &str,
) -> Result<rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream>, GatewayError> {
    let server_name = resolve_server_name(host)?;
    let connection = rustls::ClientConnection::new(build_tls_config(), server_name)
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    Ok(rustls::StreamOwned::new(connection, stream))
}

fn smtp_read_response<T: BufRead>(reader: &mut T) -> Result<(u16, String), GatewayError> {
    let mut message = String::new();
    let code = loop {
        let parsed_code;
        let continuation;
        let trimmed;
        {
            let mut line = String::new();
            let bytes = reader
                .read_line(&mut line)
                .map_err(|err| GatewayError::Internal(err.to_string()))?;
            if bytes == 0 {
                return Err(GatewayError::Internal(
                    "smtp connection closed unexpectedly".to_string(),
                ));
            }
            trimmed = line.trim_end_matches(['\r', '\n']).to_string();
            if trimmed.len() < 3 {
                return Err(GatewayError::Internal("invalid smtp response".to_string()));
            }
            parsed_code = trimmed[..3]
                .parse::<u16>()
                .map_err(|err| GatewayError::Internal(err.to_string()))?;
            continuation = trimmed.as_bytes().get(3).copied() == Some(b'-');
        }
        if !message.is_empty() {
            message.push('\n');
        }
        message.push_str(&trimmed);
        if !continuation {
            break parsed_code;
        }
    };
    Ok((code, message))
}

fn smtp_expect<T: BufRead>(reader: &mut T, allowed_codes: &[u16]) -> Result<String, GatewayError> {
    let (code, message) = smtp_read_response(reader)?;
    if allowed_codes.contains(&code) {
        return Ok(message);
    }
    Err(GatewayError::Internal(format!(
        "unexpected smtp response {code}: {message}"
    )))
}

fn smtp_write_line<T: Write>(writer: &mut T, line: &str) -> Result<(), GatewayError> {
    writer
        .write_all(line.as_bytes())
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    writer
        .write_all(b"\r\n")
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    writer
        .flush()
        .map_err(|err| GatewayError::Internal(err.to_string()))
}

fn smtp_send_command<S: std::io::Read + Write>(
    reader: &mut std::io::BufReader<S>,
    command: &str,
    allowed_codes: &[u16],
) -> Result<String, GatewayError> {
    smtp_write_line(reader.get_mut(), command)?;
    smtp_expect(reader, allowed_codes)
}

fn build_email_message(config: &SmtpConfig, email: &EmailMessage) -> String {
    let boundary = format!("niffler-{}", uuid::Uuid::new_v4().simple());
    let text_body =
        wrap_base64(&base64::engine::general_purpose::STANDARD.encode(email.text_body.as_bytes()));
    let html_body =
        wrap_base64(&base64::engine::general_purpose::STANDARD.encode(email.html_body.as_bytes()));
    let from_header = if config.from_name.trim().is_empty() {
        format!("<{}>", config.from_email)
    } else {
        format!(
            "{} <{}>",
            encode_mime_header(config.from_name.trim()),
            config.from_email
        )
    };
    format!(
        "From: {from_header}\r\nTo: <{to_email}>\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=\"{boundary}\"\r\n\r\n--{boundary}\r\nContent-Type: text/plain; charset=\"utf-8\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{text_body}--{boundary}\r\nContent-Type: text/html; charset=\"utf-8\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{html_body}--{boundary}--\r\n",
        to_email = email.to_email,
        subject = encode_mime_header(&email.subject),
    )
}

fn smtp_authenticate<S: std::io::Read + Write>(
    reader: &mut std::io::BufReader<S>,
    config: &SmtpConfig,
) -> Result<(), GatewayError> {
    let Some(username) = config
        .user
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let password = config.password.as_deref().unwrap_or("");
    smtp_send_command(reader, "AUTH LOGIN", &[334])?;
    smtp_send_command(
        reader,
        &base64::engine::general_purpose::STANDARD.encode(username.as_bytes()),
        &[334],
    )?;
    smtp_send_command(
        reader,
        &base64::engine::general_purpose::STANDARD.encode(password.as_bytes()),
        &[235],
    )?;
    Ok(())
}

fn smtp_deliver_message<S: std::io::Read + Write>(
    reader: &mut std::io::BufReader<S>,
    config: &SmtpConfig,
    email: &EmailMessage,
) -> Result<(), GatewayError> {
    smtp_send_command(
        reader,
        &format!("MAIL FROM:<{}>", config.from_email),
        &[250],
    )?;
    smtp_send_command(
        reader,
        &format!("RCPT TO:<{}>", email.to_email),
        &[250, 251],
    )?;
    smtp_send_command(reader, "DATA", &[354])?;
    let message = build_email_message(config, email);
    reader
        .get_mut()
        .write_all(message.as_bytes())
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    reader
        .get_mut()
        .write_all(b"\r\n.\r\n")
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    reader
        .get_mut()
        .flush()
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    let _ = smtp_expect(reader, &[250])?;
    let _ = smtp_send_command(reader, "QUIT", &[221]);
    Ok(())
}

fn smtp_send_message<S: std::io::Read + Write>(
    reader: &mut std::io::BufReader<S>,
    config: &SmtpConfig,
    email: &EmailMessage,
) -> Result<(), GatewayError> {
    smtp_send_command(reader, "EHLO niffler.local", &[250])?;
    smtp_authenticate(reader, config)?;
    smtp_deliver_message(reader, config, email)
}

fn smtp_probe<S: std::io::Read + Write>(
    reader: &mut std::io::BufReader<S>,
    config: &SmtpConfig,
) -> Result<(), GatewayError> {
    smtp_send_command(reader, "EHLO niffler.local", &[250])?;
    smtp_authenticate(reader, config)?;
    let _ = smtp_send_command(reader, "QUIT", &[221]);
    Ok(())
}

pub(crate) fn send_email_blocking(
    config: SmtpConfig,
    email: EmailMessage,
) -> Result<(), GatewayError> {
    if config.use_ssl {
        let stream = connect_tcp_stream(&config)?;
        let tls_stream = wrap_tls_stream(stream, &config.host)?;
        let mut reader = std::io::BufReader::new(tls_stream);
        let _ = smtp_expect(&mut reader, &[220])?;
        return smtp_send_message(&mut reader, &config, &email);
    }

    let stream = connect_tcp_stream(&config)?;
    let mut reader = std::io::BufReader::new(stream);
    let _ = smtp_expect(&mut reader, &[220])?;
    let _ = smtp_send_command(&mut reader, "EHLO niffler.local", &[250])?;
    if config.use_tls {
        let _ = smtp_send_command(&mut reader, "STARTTLS", &[220])?;
        let stream = reader.into_inner();
        let tls_stream = wrap_tls_stream(stream, &config.host)?;
        let mut reader = std::io::BufReader::new(tls_stream);
        return smtp_send_message(&mut reader, &config, &email);
    }

    smtp_authenticate(&mut reader, &config)?;
    smtp_deliver_message(&mut reader, &config, &email)
}

pub(crate) fn test_smtp_connection_blocking(config: SmtpConfig) -> SmtpTestResult {
    let result = if config.use_ssl {
        test_smtp_ssl_connection(&config)
    } else {
        test_smtp_plain_or_starttls_connection(&config)
    };
    match result {
        Ok(()) => SmtpTestResult {
            success: true,
            message: "SMTP 连接测试成功".to_string(),
        },
        Err(error) => SmtpTestResult {
            success: false,
            message: translate_smtp_error(&format!("{error:?}")),
        },
    }
}

fn test_smtp_ssl_connection(config: &SmtpConfig) -> Result<(), GatewayError> {
    let stream = connect_tcp_stream(config)?;
    let tls_stream = wrap_tls_stream(stream, &config.host)?;
    let mut reader = std::io::BufReader::new(tls_stream);
    smtp_expect(&mut reader, &[220])?;
    smtp_probe(&mut reader, config)
}

fn test_smtp_plain_or_starttls_connection(config: &SmtpConfig) -> Result<(), GatewayError> {
    let stream = connect_tcp_stream(config)?;
    let mut reader = std::io::BufReader::new(stream);
    smtp_expect(&mut reader, &[220])?;
    smtp_send_command(&mut reader, "EHLO niffler.local", &[250])?;
    if config.use_tls {
        smtp_send_command(&mut reader, "STARTTLS", &[220])?;
        let stream = reader.into_inner();
        let tls_stream = wrap_tls_stream(stream, &config.host)?;
        let mut reader = std::io::BufReader::new(tls_stream);
        return smtp_probe(&mut reader, config);
    }

    smtp_authenticate(&mut reader, config)?;
    let _ = smtp_send_command(&mut reader, "QUIT", &[221]);
    Ok(())
}

fn translate_smtp_error(error: &str) -> String {
    let error_lower = error.to_ascii_lowercase();

    if error_lower.contains("username and password not accepted") {
        return "用户名或密码错误，请检查 SMTP 凭据".to_string();
    }
    if error_lower.contains("authentication failed")
        || error_lower.contains("auth") && error_lower.contains("535")
    {
        return "认证失败，请检查用户名和密码".to_string();
    }
    if error_lower.contains("invalid credentials") || error_lower.contains("badcredentials") {
        return "凭据无效，请检查用户名和密码".to_string();
    }
    if error_lower.contains("smtp auth extension is not supported") {
        return "服务器不支持认证，请尝试使用 TLS 或 SSL 加密".to_string();
    }
    if error_lower.contains("connection refused") || error_lower.contains("os error 61") {
        return "连接被拒绝，请检查服务器地址和端口".to_string();
    }
    if error_lower.contains("connection timed out")
        || error_lower.contains("timed out")
        || error_lower.contains("operation timed out")
    {
        return "连接超时，请检查网络或服务器地址".to_string();
    }
    if error_lower.contains("name or service not known")
        || error_lower.contains("getaddrinfo failed")
        || error_lower.contains("nodename nor servname provided")
        || error_lower.contains("failed to lookup address information")
    {
        return "无法解析服务器地址，请检查 SMTP 服务器地址".to_string();
    }
    if error_lower.contains("network is unreachable") {
        return "网络不可达，请检查网络连接".to_string();
    }
    if error_lower.contains("certificate") && error_lower.contains("verify") {
        return "SSL 证书验证失败，请检查服务器证书或尝试其他加密方式".to_string();
    }
    if error_lower.contains("ssl") && error_lower.contains("wrong version") {
        return "SSL 版本不匹配，请尝试其他加密方式".to_string();
    }
    if error_lower.contains("starttls") {
        return "STARTTLS 握手失败，请检查加密设置".to_string();
    }
    if error_lower.contains("sender address rejected") {
        return "发件人地址被拒绝，请检查发件人邮箱设置".to_string();
    }
    if error_lower.contains("relay access denied") {
        return "中继访问被拒绝，请检查 SMTP 服务器配置".to_string();
    }

    error.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_common_smtp_errors() {
        assert_eq!(
            translate_smtp_error("connection refused"),
            "连接被拒绝，请检查服务器地址和端口"
        );
        assert_eq!(
            translate_smtp_error("535 authentication failed"),
            "认证失败，请检查用户名和密码"
        );
        assert_eq!(
            translate_smtp_error("nodename nor servname provided"),
            "无法解析服务器地址，请检查 SMTP 服务器地址"
        );
    }
}

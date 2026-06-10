use crate::GatewayError;
use crate::email::{SmtpConfig, test_smtp_connection_blocking};
use crate::handlers::admin::request::AdminAppState;
use crate::handlers::shared::{system_config_bool, system_config_string};
use axum::body::Bytes;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Default, Deserialize)]
struct AdminSmtpTestRequest {
    smtp_host: Option<serde_json::Value>,
    smtp_port: Option<serde_json::Value>,
    smtp_user: Option<serde_json::Value>,
    smtp_password: Option<serde_json::Value>,
    smtp_use_tls: Option<serde_json::Value>,
    smtp_use_ssl: Option<serde_json::Value>,
    smtp_from_email: Option<serde_json::Value>,
    smtp_from_name: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct ResolvedSmtpConfig {
    host: Option<String>,
    port: u16,
    user: Option<String>,
    password: Option<String>,
    use_tls: bool,
    use_ssl: bool,
    from_email: Option<String>,
    #[allow(dead_code)]
    from_name: String,
}

pub(crate) async fn build_admin_smtp_test_payload(
    state: &AdminAppState<'_>,
    request_body: Option<&Bytes>,
) -> Result<serde_json::Value, GatewayError> {
    let request = match request_body {
        Some(body) if !body.is_empty() => serde_json::from_slice::<AdminSmtpTestRequest>(body)
            .map_err(|err| GatewayError::Internal(err.to_string()))?,
        _ => AdminSmtpTestRequest::default(),
    };
    let config = resolve_admin_smtp_config(state, request).await?;
    let missing_fields = missing_smtp_fields(&config);
    if !missing_fields.is_empty() {
        return Ok(json!({
            "success": false,
            "message": format!("SMTP 配置不完整，请检查 {}", missing_fields.join(", ")),
        }));
    }

    let smtp_config = config.into_smtp_config();
    let result = tokio::task::spawn_blocking(move || test_smtp_connection_blocking(smtp_config))
        .await
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    Ok(json!({
        "success": result.success,
        "message": result.message,
    }))
}

async fn resolve_admin_smtp_config(
    state: &AdminAppState<'_>,
    request: AdminSmtpTestRequest,
) -> Result<ResolvedSmtpConfig, GatewayError> {
    let smtp_host = state.read_system_config_json_value("smtp_host").await?;
    let smtp_port = state.read_system_config_json_value("smtp_port").await?;
    let smtp_user = state.read_system_config_json_value("smtp_user").await?;
    let smtp_password = state.read_system_config_json_value("smtp_password").await?;
    let smtp_use_tls = state.read_system_config_json_value("smtp_use_tls").await?;
    let smtp_use_ssl = state.read_system_config_json_value("smtp_use_ssl").await?;
    let smtp_from_email = state
        .read_system_config_json_value("smtp_from_email")
        .await?;
    let smtp_from_name = state
        .read_system_config_json_value("smtp_from_name")
        .await?;

    let stored_password = system_config_string(smtp_password.as_ref()).map(|value| {
        state
            .decrypt_catalog_secret_with_fallbacks(&value)
            .unwrap_or(value)
    });

    Ok(ResolvedSmtpConfig {
        host: request
            .smtp_host
            .as_ref()
            .and_then(|value| system_config_string(Some(value)))
            .or_else(|| system_config_string(smtp_host.as_ref())),
        port: request
            .smtp_port
            .as_ref()
            .map(|value| system_config_u16(value, 587))
            .unwrap_or_else(|| system_config_u16_opt(smtp_port.as_ref(), 587)),
        user: request
            .smtp_user
            .as_ref()
            .and_then(|value| system_config_string(Some(value)))
            .or_else(|| system_config_string(smtp_user.as_ref())),
        password: request
            .smtp_password
            .as_ref()
            .and_then(|value| system_config_string(Some(value)))
            .or(stored_password),
        use_tls: request
            .smtp_use_tls
            .as_ref()
            .map(|value| system_config_bool(Some(value), true))
            .unwrap_or_else(|| system_config_bool(smtp_use_tls.as_ref(), true)),
        use_ssl: request
            .smtp_use_ssl
            .as_ref()
            .map(|value| system_config_bool(Some(value), false))
            .unwrap_or_else(|| system_config_bool(smtp_use_ssl.as_ref(), false)),
        from_email: request
            .smtp_from_email
            .as_ref()
            .and_then(|value| system_config_string(Some(value)))
            .or_else(|| system_config_string(smtp_from_email.as_ref())),
        from_name: request
            .smtp_from_name
            .as_ref()
            .and_then(|value| system_config_string(Some(value)))
            .or_else(|| system_config_string(smtp_from_name.as_ref()))
            .unwrap_or_else(|| "Niffler".to_string()),
    })
}

impl ResolvedSmtpConfig {
    fn into_smtp_config(self) -> SmtpConfig {
        SmtpConfig {
            host: self.host.unwrap_or_default(),
            port: self.port,
            user: self.user,
            password: self.password,
            use_tls: self.use_tls,
            use_ssl: self.use_ssl,
            from_email: self.from_email.unwrap_or_default(),
            from_name: self.from_name,
        }
    }
}

fn missing_smtp_fields(config: &ResolvedSmtpConfig) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if config
        .host
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        fields.push("smtp_host");
    }
    if config
        .user
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        fields.push("smtp_user");
    }
    if config
        .password
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        fields.push("smtp_password");
    }
    if config
        .from_email
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        fields.push("smtp_from_email");
    }
    fields
}

fn system_config_u16_opt(value: Option<&serde_json::Value>, default: u16) -> u16 {
    value
        .map(|value| system_config_u16(value, default))
        .unwrap_or(default)
}

fn system_config_u16(value: &serde_json::Value, default: u16) -> u16 {
    match value {
        serde_json::Value::Number(value) => value
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(default),
        serde_json::Value::String(value) => value.trim().parse::<u16>().unwrap_or(default),
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_missing_python_required_fields() {
        let config = ResolvedSmtpConfig {
            host: None,
            port: 587,
            user: None,
            password: None,
            use_tls: true,
            use_ssl: false,
            from_email: None,
            from_name: "Niffler".to_string(),
        };
        assert_eq!(
            missing_smtp_fields(&config),
            vec!["smtp_host", "smtp_user", "smtp_password", "smtp_from_email"]
        );
    }
}

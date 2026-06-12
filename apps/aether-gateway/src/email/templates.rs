use std::collections::BTreeMap;

use super::EmailDeliveryPayload;
use crate::handlers::shared::{
    escape_admin_email_template_html, read_admin_email_template_payload,
    render_admin_email_template_html, system_config_string,
};
use crate::{AppState, GatewayError};

pub(crate) async fn build_verification_email_payload(
    state: &AppState,
    email: &str,
    code: &str,
    expire_minutes: i64,
) -> Result<EmailDeliveryPayload, GatewayError> {
    let template = read_admin_email_template_payload(state, "verification")
        .await?
        .ok_or_else(|| GatewayError::Internal("verification email template missing".to_string()))?;
    let subject_template = template
        .get("subject")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("邮箱验证码");
    let html_template = template
        .get("html")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let app_name = email_app_name(state).await?;
    let variables = BTreeMap::from([
        ("app_name".to_string(), app_name.clone()),
        ("code".to_string(), code.to_string()),
        ("expire_minutes".to_string(), expire_minutes.to_string()),
        ("email".to_string(), email.to_string()),
    ]);
    let subject = render_template_string(subject_template, &variables, false)?;
    let html_body = render_admin_email_template_html(html_template, &variables)?;
    let text_body = format!(
        "{app_name}\n\n您的验证码是：{code}\n目标邮箱：{email}\n有效期：{expire_minutes} 分钟\n\n如果这不是您本人的操作，请忽略此邮件。"
    );
    Ok(EmailDeliveryPayload {
        message_type: "verification".to_string(),
        to_email: email.to_string(),
        subject,
        html_body,
        text_body,
    })
}

pub(crate) async fn build_test_email_payload(
    state: &AppState,
    email: &str,
) -> Result<EmailDeliveryPayload, GatewayError> {
    let app_name = email_app_name(state).await?;
    Ok(EmailDeliveryPayload {
        message_type: "test".to_string(),
        to_email: email.to_string(),
        subject: format!("{app_name} 测试邮件"),
        html_body: format!(
            "<!doctype html><html><body><p>{}</p><p>这是一封测试邮件。如果您收到这封邮件，说明邮件发送服务可以正常使用。</p></body></html>",
            escape_admin_email_template_html(&app_name)
        ),
        text_body: format!(
            "{app_name}\n\n这是一封测试邮件。如果您收到这封邮件，说明邮件发送服务可以正常使用。"
        ),
    })
}

pub(crate) async fn build_password_reset_email_payload(
    state: &AppState,
    email: &str,
    reset_link: &str,
    expire_minutes: i64,
) -> Result<EmailDeliveryPayload, GatewayError> {
    let template = read_admin_email_template_payload(state, "password_reset")
        .await?
        .ok_or_else(|| {
            GatewayError::Internal("password reset email template missing".to_string())
        })?;
    let subject_template = template
        .get("subject")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("密码重置");
    let html_template = template
        .get("html")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let app_name = email_app_name(state).await?;
    let variables = BTreeMap::from([
        ("app_name".to_string(), app_name.clone()),
        ("reset_link".to_string(), reset_link.to_string()),
        ("expire_minutes".to_string(), expire_minutes.to_string()),
        ("email".to_string(), email.to_string()),
    ]);
    let subject = render_template_string(subject_template, &variables, false)?;
    let html_body = render_admin_email_template_html(html_template, &variables)?;
    let text_body = format!(
        "{app_name}\n\n请打开以下链接重置密码：\n{reset_link}\n\n目标邮箱：{email}\n链接有效期：{expire_minutes} 分钟\n\n如果这不是您本人的操作，请忽略此邮件。"
    );
    Ok(EmailDeliveryPayload {
        message_type: "password_reset".to_string(),
        to_email: email.to_string(),
        subject,
        html_body,
        text_body,
    })
}

async fn email_app_name(state: &AppState) -> Result<String, GatewayError> {
    let email_app_name = state
        .read_system_config_json_value("email_app_name")
        .await?;
    let site_name = state.read_system_config_json_value("site_name").await?;
    let smtp_from_name = state
        .read_system_config_json_value("smtp_from_name")
        .await?;
    Ok(system_config_string(email_app_name.as_ref())
        .or_else(|| system_config_string(site_name.as_ref()))
        .or_else(|| system_config_string(smtp_from_name.as_ref()))
        .unwrap_or_else(|| "Niffler".to_string()))
}

fn render_template_string(
    template: &str,
    variables: &BTreeMap<String, String>,
    escape_html: bool,
) -> Result<String, GatewayError> {
    let mut rendered = template.to_string();
    for (key, value) in variables {
        let pattern = regex::Regex::new(&format!(r"\{{\{{\s*{}\s*\}}\}}", regex::escape(key)))
            .map_err(|err| GatewayError::Internal(err.to_string()))?;
        let replacement = if escape_html {
            escape_admin_email_template_html(value)
        } else {
            value.clone()
        };
        rendered = pattern
            .replace_all(&rendered, replacement.as_str())
            .into_owned();
    }
    Ok(rendered)
}

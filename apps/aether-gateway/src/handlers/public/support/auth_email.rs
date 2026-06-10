use super::{
    json, system_config_string, AppState, GatewayError, AUTH_EMAIL_VERIFICATION_PREFIX,
    AUTH_EMAIL_VERIFIED_PREFIX, AUTH_EMAIL_VERIFIED_TTL_SECS,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct StoredAuthEmailVerificationCode {
    pub(super) code: String,
    pub(super) created_at: String,
    #[serde(default)]
    pub(super) delivery_id: Option<String>,
}

pub(super) fn auth_email_verification_key(email: &str) -> String {
    format!("{AUTH_EMAIL_VERIFICATION_PREFIX}{email}")
}

pub(super) fn auth_email_verified_key(email: &str) -> String {
    format!("{AUTH_EMAIL_VERIFIED_PREFIX}{email}")
}

pub(super) fn generate_auth_verification_code() -> String {
    format!("{:06}", uuid::Uuid::new_v4().as_u128() % 1_000_000)
}

pub(super) async fn read_auth_email_verification_code(
    state: &AppState,
    email: &str,
) -> Result<Option<StoredAuthEmailVerificationCode>, GatewayError> {
    let key = auth_email_verification_key(email);
    let raw = state.runtime_kv_get(&key).await?;
    raw.map(|value| {
        serde_json::from_str::<StoredAuthEmailVerificationCode>(&value)
            .map_err(|err| GatewayError::Internal(err.to_string()))
    })
    .transpose()
}

pub(super) async fn auth_email_is_verified(
    state: &AppState,
    email: &str,
) -> Result<bool, GatewayError> {
    let key = auth_email_verified_key(email);
    state.runtime_kv_exists(&key).await
}

pub(super) async fn mark_auth_email_verified(
    state: &AppState,
    email: &str,
) -> Result<bool, GatewayError> {
    let key = auth_email_verified_key(email);
    state
        .runtime_kv_setex(&key, "verified", AUTH_EMAIL_VERIFIED_TTL_SECS)
        .await?;
    Ok(true)
}

pub(super) async fn clear_auth_email_pending_code(
    state: &AppState,
    email: &str,
) -> Result<bool, GatewayError> {
    let verification_key = auth_email_verification_key(email);
    state.runtime_kv_del(&verification_key).await
}

pub(super) async fn clear_auth_email_verification(
    state: &AppState,
    email: &str,
) -> Result<bool, GatewayError> {
    let verification_key = auth_email_verification_key(email);
    let verified_key = auth_email_verified_key(email);
    let deleted_pending = state.runtime_kv_del(&verification_key).await?;
    let deleted_verified = state.runtime_kv_del(&verified_key).await?;
    Ok(deleted_pending || deleted_verified)
}

pub(super) async fn store_auth_email_verification_code_with_delivery(
    state: &AppState,
    email: &str,
    code: &str,
    created_at: chrono::DateTime<chrono::Utc>,
    ttl_seconds: u64,
    delivery_id: Option<&str>,
) -> Result<bool, GatewayError> {
    let key = auth_email_verification_key(email);
    let value = json!({
        "code": code,
        "created_at": created_at.to_rfc3339(),
        "delivery_id": delivery_id,
    })
    .to_string();
    state.runtime_kv_setex(&key, &value, ttl_seconds).await?;
    Ok(true)
}

pub(super) fn smtp_config_is_complete(
    host: Option<&serde_json::Value>,
    user: Option<&serde_json::Value>,
    password: Option<&serde_json::Value>,
    from_email: Option<&serde_json::Value>,
) -> bool {
    system_config_string(host).is_some()
        && system_config_string(user).is_some()
        && system_config_string(password).is_some()
        && system_config_string(from_email).is_some()
}

pub(super) async fn auth_email_app_name(state: &AppState) -> Result<String, GatewayError> {
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

pub(super) async fn auth_registration_email_configured(
    state: &AppState,
) -> Result<bool, GatewayError> {
    let smtp_host = state.read_system_config_json_value("smtp_host").await?;
    let smtp_user = state.read_system_config_json_value("smtp_user").await?;
    let smtp_password = state.read_system_config_json_value("smtp_password").await?;
    let smtp_from_email = state
        .read_system_config_json_value("smtp_from_email")
        .await?;
    Ok(smtp_config_is_complete(
        smtp_host.as_ref(),
        smtp_user.as_ref(),
        smtp_password.as_ref(),
        smtp_from_email.as_ref(),
    ))
}

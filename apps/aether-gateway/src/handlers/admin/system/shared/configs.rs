use crate::content_moderation::CONTENT_MODERATION_CONFIG_KEY;
use crate::handlers::admin::request::AdminAppState;
use crate::handlers::admin::shared::{take_secret_prefix, take_secret_suffix};
use crate::handlers::shared::unix_secs_to_rfc3339;
use crate::GatewayError;
use aether_admin::system::{
    admin_system_config_default_value as admin_system_config_default_value_pure,
    admin_system_config_delete_keys as admin_system_config_delete_keys_pure,
    build_admin_system_config_deleted_payload,
    build_admin_system_config_detail_payload as build_admin_system_config_detail_payload_pure,
    build_admin_system_config_updated_payload,
    build_admin_system_configs_payload as build_admin_system_configs_payload_pure,
    build_content_moderation_config_admin_value,
    is_sensitive_admin_system_config_key as is_sensitive_admin_system_config_key_pure,
    normalize_admin_system_config_key as normalize_admin_system_config_key_pure,
    parse_admin_system_config_update,
};
use aether_crypto::encrypt_python_fernet_plaintext;
use axum::body::Bytes;
use axum::http;
use serde_json::{json, Value};

fn normalize_admin_system_config_key(requested_key: &str) -> String {
    normalize_admin_system_config_key_pure(requested_key)
}

fn admin_system_config_delete_keys(requested_key: &str) -> Vec<String> {
    admin_system_config_delete_keys_pure(requested_key)
}

pub(crate) fn is_sensitive_admin_system_config_key(key: &str) -> bool {
    is_sensitive_admin_system_config_key_pure(key)
}

fn admin_system_config_default_value(key: &str) -> Option<serde_json::Value> {
    admin_system_config_default_value_pure(key)
}

pub(crate) fn build_admin_system_configs_payload(
    state: &AdminAppState<'_>,
    entries: &[aether_data::repository::system::StoredSystemConfigEntry],
) -> serde_json::Value {
    let visible_entries = entries
        .iter()
        .cloned()
        .map(|mut entry| {
            if is_content_moderation_config_key(entry.key.as_str()) {
                entry.value = content_moderation_admin_display_value(state, &entry.value);
            }
            entry
        })
        .collect::<Vec<_>>();
    build_admin_system_configs_payload_pure(&visible_entries)
}

pub(crate) async fn build_admin_system_config_detail_payload(
    state: &AdminAppState<'_>,
    requested_key: &str,
) -> Result<Result<serde_json::Value, (http::StatusCode, serde_json::Value)>, GatewayError> {
    let requested_key = requested_key.trim();
    let value = state
        .read_system_config_json_value(&normalize_admin_system_config_key(requested_key))
        .await?
        .or_else(|| {
            admin_system_config_default_value(&normalize_admin_system_config_key(requested_key))
        });
    let value = if normalize_admin_system_config_key(requested_key) == CONTENT_MODERATION_CONFIG_KEY
    {
        value.map(|value| content_moderation_admin_display_value(state, &value))
    } else {
        value
    };
    Ok(build_admin_system_config_detail_payload_pure(
        requested_key,
        value,
    ))
}

pub(crate) async fn apply_admin_system_config_update(
    state: &AdminAppState<'_>,
    requested_key: &str,
    request_body: &Bytes,
) -> Result<Result<serde_json::Value, (http::StatusCode, serde_json::Value)>, GatewayError> {
    let update = match parse_admin_system_config_update(requested_key, request_body) {
        Ok(update) => update,
        Err(err) => return Ok(Err(err)),
    };
    let mut value = update.value;
    let normalized_key = update.normalized_key;
    let description = update.description;

    if normalized_key == CONTENT_MODERATION_CONFIG_KEY {
        let existing = state.read_system_config_json_value(&normalized_key).await?;
        value = match prepare_content_moderation_config_for_storage(state, value, existing.as_ref())
        {
            Ok(value) => value,
            Err(err) => return Ok(Err(err)),
        };
    } else if is_sensitive_admin_system_config_key(&normalized_key)
        && value.as_str().is_some_and(|raw| !raw.is_empty())
    {
        let Some(encryption_key) = state
            .encryption_key()
            .filter(|value| !value.trim().is_empty())
        else {
            return Ok(Err((
                http::StatusCode::SERVICE_UNAVAILABLE,
                json!({ "detail": "系统配置写入需要可用的加密密钥" }),
            )));
        };
        let plaintext = value.as_str().unwrap();
        value = json!(encrypt_python_fernet_plaintext(encryption_key, plaintext)
            .map_err(|err| GatewayError::Internal(err.to_string()))?);
    }

    let updated = state
        .upsert_system_config_entry(&normalized_key, &value, description.as_deref())
        .await?;
    let display_value = if normalized_key == CONTENT_MODERATION_CONFIG_KEY {
        content_moderation_admin_display_value(state, &updated.value)
    } else if is_sensitive_admin_system_config_key(&normalized_key) {
        json!("********")
    } else {
        updated.value.clone()
    };
    Ok(Ok(build_admin_system_config_updated_payload(
        updated.key,
        display_value,
        updated.description,
        updated.updated_at_unix_secs,
    )))
}

fn is_content_moderation_config_key(key: &str) -> bool {
    key.eq_ignore_ascii_case(CONTENT_MODERATION_CONFIG_KEY)
}

fn content_moderation_admin_display_value(
    state: &AdminAppState<'_>,
    value: &Value,
) -> serde_json::Value {
    let Value::Object(object) = value else {
        return build_content_moderation_config_admin_value(value);
    };
    let mut visible = object.clone();
    let encrypted_items = object
        .get("api_keys_encrypted")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let plaintext_keys = content_moderation_plain_api_keys_from_value(value);
    let masks = if encrypted_items.is_empty() {
        plaintext_keys
            .iter()
            .map(|key| mask_content_moderation_api_key(key))
            .collect::<Vec<_>>()
    } else {
        encrypted_items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let ciphertext = item.as_str()?.trim();
                if ciphertext.is_empty() {
                    return None;
                }
                Some(
                    state
                        .decrypt_catalog_secret_with_fallbacks(ciphertext)
                        .map(|key| mask_content_moderation_api_key(&key))
                        .unwrap_or_else(|| format!("已保存 Key {}", index + 1)),
                )
            })
            .collect::<Vec<_>>()
    };
    let key_count = encrypted_items.len().max(plaintext_keys.len());
    visible.remove("api_keys");
    visible.remove("api_keys_encrypted");
    visible.remove("api_keys_clear");
    visible.insert("api_keys".to_string(), json!([]));
    visible.insert("api_key_count".to_string(), json!(key_count));
    visible.insert("api_key_masks".to_string(), json!(masks));
    build_content_moderation_config_admin_value(&Value::Object(visible))
}

pub(crate) fn content_moderation_admin_export_value(
    state: &AdminAppState<'_>,
    value: &Value,
) -> serde_json::Value {
    let Value::Object(object) = value else {
        return value.clone();
    };
    let mut exported = object.clone();
    let mut api_keys = content_moderation_plain_api_keys_from_value(value);
    if api_keys.is_empty() {
        api_keys = object
            .get("api_keys_encrypted")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .filter_map(|ciphertext| {
                        state.decrypt_catalog_secret_with_fallbacks(ciphertext.trim())
                    })
                    .filter(|api_key| !api_key.trim().is_empty())
                    .fold(Vec::<String>::new(), |mut keys, api_key| {
                        if !keys.iter().any(|existing| existing == &api_key) {
                            keys.push(api_key);
                        }
                        keys
                    })
            })
            .unwrap_or_default();
    }
    exported.remove("api_key_count");
    exported.remove("api_key_masks");
    exported.remove("api_keys_encrypted");
    exported.remove("api_keys_clear");
    exported.insert("api_keys".to_string(), json!(api_keys));
    Value::Object(exported)
}

fn prepare_content_moderation_config_for_storage(
    state: &AdminAppState<'_>,
    mut value: Value,
    existing: Option<&Value>,
) -> Result<Value, (http::StatusCode, serde_json::Value)> {
    let Some(object) = value.as_object_mut() else {
        return Err((
            http::StatusCode::BAD_REQUEST,
            json!({ "detail": "请求数据验证失败" }),
        ));
    };
    let new_plain_keys =
        content_moderation_plain_api_keys_from_value(&Value::Object(object.clone()));
    let clear_api_keys = object
        .get("api_keys_clear")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    object.remove("api_key_count");
    object.remove("api_key_masks");
    object.remove("api_keys");
    object.remove("api_keys_encrypted");
    object.remove("api_keys_clear");

    let encrypted_keys = if clear_api_keys {
        None
    } else if new_plain_keys.is_empty() {
        existing
            .and_then(content_moderation_encrypted_api_keys_from_value)
            .or_else(|| {
                let existing_plain_keys = existing
                    .map(content_moderation_plain_api_keys_from_value)
                    .unwrap_or_default();
                if existing_plain_keys.is_empty() {
                    None
                } else {
                    Some(encrypt_content_moderation_api_keys(
                        state,
                        &existing_plain_keys,
                    ))
                }
            })
            .transpose()?
    } else {
        Some(encrypt_content_moderation_api_keys(state, &new_plain_keys)?)
    };
    if let Some(encrypted_keys) = encrypted_keys {
        if !encrypted_keys.is_empty() {
            object.insert("api_keys_encrypted".to_string(), json!(encrypted_keys));
        }
    }
    Ok(value)
}

fn content_moderation_plain_api_keys_from_value(value: &Value) -> Vec<String> {
    let Some(items) = value.get("api_keys").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut keys = Vec::new();
    for item in items {
        let Some(key) = item.as_str().map(str::trim).filter(|key| !key.is_empty()) else {
            continue;
        };
        if keys.iter().any(|existing| existing == key) {
            continue;
        }
        keys.push(key.to_string());
    }
    keys
}

fn content_moderation_encrypted_api_keys_from_value(
    value: &Value,
) -> Option<Result<Vec<String>, (http::StatusCode, serde_json::Value)>> {
    let items = value.get("api_keys_encrypted")?.as_array()?;
    let keys = items
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    Some(Ok(keys))
}

fn encrypt_content_moderation_api_keys(
    state: &AdminAppState<'_>,
    api_keys: &[String],
) -> Result<Vec<String>, (http::StatusCode, serde_json::Value)> {
    let mut encrypted = Vec::with_capacity(api_keys.len());
    for api_key in api_keys {
        let Some(ciphertext) = state.encrypt_catalog_secret_with_fallbacks(api_key) else {
            return Err((
                http::StatusCode::SERVICE_UNAVAILABLE,
                json!({ "detail": "内容审查 Key 写入需要可用的加密密钥" }),
            ));
        };
        encrypted.push(ciphertext);
    }
    Ok(encrypted)
}

fn mask_content_moderation_api_key(value: &str) -> String {
    if value.chars().count() <= 12 {
        format!("{value}***")
    } else {
        format!(
            "{}***{}",
            take_secret_prefix(value, 8),
            take_secret_suffix(value, 4)
        )
    }
}

pub(crate) async fn delete_admin_system_config(
    state: &AdminAppState<'_>,
    requested_key: &str,
) -> Result<Result<serde_json::Value, (http::StatusCode, serde_json::Value)>, GatewayError> {
    let delete_keys = admin_system_config_delete_keys(requested_key);
    let mut deleted = false;
    for key in &delete_keys {
        deleted |= state.delete_system_config_value(key).await?;
    }
    if !deleted {
        return Ok(Err((
            http::StatusCode::NOT_FOUND,
            json!({ "detail": format!("配置项 '{requested_key}' 不存在") }),
        )));
    }
    Ok(Ok(build_admin_system_config_deleted_payload(requested_key)))
}

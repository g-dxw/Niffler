use super::{
    auth_now, auth_password_policy_level, auth_password_reset_expire_minutes,
    auth_password_reset_send_cooldown_seconds, auth_registration_email_configured,
    base_url_from_request, build_auth_error_response, build_auth_json_response, http, json,
    validate_auth_register_password, AppState, Body, GatewayPublicRequestContext, Regex, Response,
    AUTH_PASSWORD_RESET_PREFIX,
};
use serde::Deserialize;
use tracing::warn;

const PASSWORD_RESET_SUCCESS_MESSAGE: &str = "如果该邮箱存在，我们会发送一封重置密码邮件";
const PASSWORD_RESET_INVALID_TOKEN_MESSAGE: &str = "重置链接无效或已过期，请重新申请";

#[derive(Debug, Deserialize)]
struct AuthRequestPasswordResetRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct AuthResetPasswordRequest {
    token: String,
    password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StoredPasswordResetToken {
    user_id: String,
    email: String,
    created_at: String,
}

pub(super) async fn handle_auth_request_password_reset(
    state: &AppState,
    request_context: &GatewayPublicRequestContext,
    headers: &http::HeaderMap,
    request_body: Option<&axum::body::Bytes>,
) -> Response<Body> {
    let Some(request_body) = request_body else {
        return build_auth_error_response(
            http::StatusCode::BAD_REQUEST,
            "缺少找回密码请求体",
            false,
        );
    };
    let payload = match serde_json::from_slice::<AuthRequestPasswordResetRequest>(request_body) {
        Ok(value) => value,
        Err(_) => {
            return build_auth_error_response(
                http::StatusCode::BAD_REQUEST,
                "无效的找回密码请求",
                false,
            )
        }
    };
    let Some(email) = normalize_password_reset_email(&payload.email) else {
        return build_auth_error_response(http::StatusCode::BAD_REQUEST, "邮箱格式无效", false);
    };
    match auth_registration_email_configured(state).await {
        Ok(true) => {}
        Ok(false) => {
            return build_auth_error_response(
                http::StatusCode::SERVICE_UNAVAILABLE,
                "邮件服务未配置，暂不能找回密码",
                false,
            )
        }
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth email settings lookup failed: {err:?}"),
                false,
            )
        }
    }

    if let Err(response) = enforce_password_reset_request_cooldown(state, &email).await {
        return response;
    }

    match state.find_user_auth_by_identifier(&email).await {
        Ok(Some(user)) if password_reset_allowed_for_user(&user) => {
            let expire_minutes = auth_password_reset_expire_minutes();
            let token = generate_password_reset_token();
            let key = password_reset_token_key(&token);
            let created_at = auth_now();
            let stored = StoredPasswordResetToken {
                user_id: user.id.clone(),
                email: email.clone(),
                created_at: created_at.to_rfc3339(),
            };
            let value = match serde_json::to_string(&stored) {
                Ok(value) => value,
                Err(err) => {
                    return build_auth_error_response(
                        http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("auth password reset token serialize failed: {err:?}"),
                        false,
                    )
                }
            };
            if let Err(err) = state
                .runtime_kv_setex(&key, &value, expire_minutes.max(1) as u64 * 60)
                .await
            {
                warn!(error = ?err, "auth password reset token save failed");
            } else {
                let reset_link = format!(
                    "{}/reset-password?token={}",
                    base_url_from_request(headers, request_context).trim_end_matches('/'),
                    token
                );
                let email_payload = match crate::email::build_password_reset_email_payload(
                    state,
                    &email,
                    &reset_link,
                    expire_minutes,
                )
                .await
                {
                    Ok(payload) => payload,
                    Err(err) => {
                        let _ = state.runtime_kv_del(&key).await;
                        warn!(error = ?err, "auth password reset email render failed");
                        return build_password_reset_request_success_response();
                    }
                };
                if let Err(err) = crate::email::queue_email_delivery(
                    state,
                    email_payload,
                    Some("auth:request_password_reset".to_string()),
                )
                .await
                {
                    let _ = state.runtime_kv_del(&key).await;
                    warn!(error = ?err, "auth password reset email queue failed");
                }
            }
        }
        Ok(_) => {}
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth user lookup failed: {err:?}"),
                false,
            )
        }
    }

    build_password_reset_request_success_response()
}

fn build_password_reset_request_success_response() -> Response<Body> {
    build_auth_json_response(
        http::StatusCode::OK,
        json!({
            "success": true,
            "message": PASSWORD_RESET_SUCCESS_MESSAGE,
        }),
        None,
    )
}

pub(super) async fn handle_auth_reset_password(
    state: &AppState,
    request_body: Option<&axum::body::Bytes>,
) -> Response<Body> {
    let Some(request_body) = request_body else {
        return build_auth_error_response(
            http::StatusCode::BAD_REQUEST,
            "缺少重置密码请求体",
            false,
        );
    };
    let payload = match serde_json::from_slice::<AuthResetPasswordRequest>(request_body) {
        Ok(value) => value,
        Err(_) => {
            return build_auth_error_response(
                http::StatusCode::BAD_REQUEST,
                "无效的重置密码请求",
                false,
            )
        }
    };
    let token = payload.token.trim();
    if token.is_empty() {
        return build_auth_error_response(
            http::StatusCode::BAD_REQUEST,
            PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
            false,
        );
    }
    let token_key = password_reset_token_key(token);
    let stored = match state.runtime_kv_get(&token_key).await {
        Ok(Some(value)) => match serde_json::from_str::<StoredPasswordResetToken>(&value) {
            Ok(stored) => stored,
            Err(_) => {
                let _ = state.runtime_kv_del(&token_key).await;
                return build_auth_error_response(
                    http::StatusCode::BAD_REQUEST,
                    PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
                    false,
                );
            }
        },
        Ok(None) => {
            return build_auth_error_response(
                http::StatusCode::BAD_REQUEST,
                PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
                false,
            )
        }
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth password reset token lookup failed: {err:?}"),
                false,
            )
        }
    };

    let user = match state.find_user_auth_by_id(&stored.user_id).await {
        Ok(Some(user)) if password_reset_allowed_for_user(&user) => user,
        Ok(_) => {
            return build_auth_error_response(
                http::StatusCode::BAD_REQUEST,
                PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
                false,
            )
        }
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth user lookup failed: {err:?}"),
                false,
            )
        }
    };
    if user.email.as_deref() != Some(stored.email.as_str()) {
        return build_auth_error_response(
            http::StatusCode::BAD_REQUEST,
            PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
            false,
        );
    }

    let policy = match auth_password_policy_level(state).await {
        Ok(value) => value,
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth password policy lookup failed: {err:?}"),
                false,
            )
        }
    };
    if let Err(detail) = validate_auth_register_password(&payload.password, &policy) {
        return build_auth_error_response(http::StatusCode::BAD_REQUEST, detail, false);
    }
    let password_hash = match bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST) {
        Ok(value) => value,
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth password hash failed: {err:?}"),
                false,
            )
        }
    };
    let consumed = match state.runtime_kv_getdel(&token_key).await {
        Ok(Some(value)) => match serde_json::from_str::<StoredPasswordResetToken>(&value) {
            Ok(stored) => stored,
            Err(_) => {
                return build_auth_error_response(
                    http::StatusCode::BAD_REQUEST,
                    PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
                    false,
                )
            }
        },
        Ok(None) => {
            return build_auth_error_response(
                http::StatusCode::BAD_REQUEST,
                PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
                false,
            )
        }
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth password reset token consume failed: {err:?}"),
                false,
            )
        }
    };
    if consumed.user_id != stored.user_id || consumed.email != stored.email {
        return build_auth_error_response(
            http::StatusCode::BAD_REQUEST,
            PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
            false,
        );
    }
    let updated_at = auth_now();
    match state
        .reset_local_auth_user_password(&user.id, password_hash, updated_at, "password_reset")
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return build_auth_error_response(
                http::StatusCode::BAD_REQUEST,
                PASSWORD_RESET_INVALID_TOKEN_MESSAGE,
                false,
            )
        }
        Err(err) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth password update failed: {err:?}"),
                false,
            )
        }
    }

    build_auth_json_response(
        http::StatusCode::OK,
        json!({
            "success": true,
            "message": "密码已重置，请使用新密码登录",
        }),
        None,
    )
}

fn normalize_password_reset_email(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
    let pattern = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("email regex should compile");
    pattern.is_match(&value).then_some(value)
}

fn password_reset_token_key(token: &str) -> String {
    format!("{AUTH_PASSWORD_RESET_PREFIX}{token}")
}

fn password_reset_cooldown_key(email: &str) -> String {
    format!("{AUTH_PASSWORD_RESET_PREFIX}cooldown:{email}")
}

async fn enforce_password_reset_request_cooldown(
    state: &AppState,
    email: &str,
) -> Result<(), Response<Body>> {
    let cooldown_seconds = auth_password_reset_send_cooldown_seconds();
    if cooldown_seconds <= 0 {
        return Ok(());
    }
    let key = password_reset_cooldown_key(email);
    let now = auth_now();
    match state.runtime_kv_get(&key).await {
        Ok(Some(created_at)) => {
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
                .ok()
                .map(|value| value.with_timezone(&chrono::Utc));
            if let Some(created_at) = created_at {
                let elapsed = now.signed_duration_since(created_at).num_seconds();
                let remaining = cooldown_seconds - elapsed;
                if remaining > 0 {
                    return Err(build_auth_error_response(
                        http::StatusCode::BAD_REQUEST,
                        format!("请在 {remaining} 秒后重试"),
                        false,
                    ));
                }
            }
            let _ = state.runtime_kv_del(&key).await;
        }
        Ok(None) => {}
        Err(err) => {
            return Err(build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth password reset cooldown lookup failed: {err:?}"),
                false,
            ))
        }
    }

    state
        .runtime_kv_setex(&key, &now.to_rfc3339(), cooldown_seconds as u64)
        .await
        .map_err(|err| {
            build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("auth password reset cooldown save failed: {err:?}"),
                false,
            )
        })
}

fn generate_password_reset_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn password_reset_allowed_for_user(
    user: &aether_data::repository::users::StoredUserAuthRecord,
) -> bool {
    !user.is_deleted
        && user.is_active
        && user.auth_source.eq_ignore_ascii_case("local")
        && user
            .email
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        && user
            .password_hash
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
}

use axum::{body::Body, http, response::Response};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use tracing::warn;
use uuid::Uuid;

use super::{payment_shared::payment_callback_payload_hash, AppState, GatewayPublicRequestContext};

#[derive(Debug, Clone)]
pub(crate) struct DodopayConfig {
    pub(crate) base_url: String,
    pub(crate) product_id: String,
    pub(crate) api_key: String,
    pub(crate) webhook_secret: Option<String>,
    pub(crate) callback_base_url: Option<String>,
    pub(crate) return_path: String,
    pub(crate) pay_currency: String,
    pub(crate) usd_exchange_rate: f64,
    pub(crate) min_recharge_usd: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct DodopayCheckoutInput {
    pub(crate) order_no: String,
    pub(crate) subject: String,
    pub(crate) pay_amount: f64,
    pub(crate) return_url: String,
    pub(crate) cancel_url: String,
    pub(crate) payment_channel: String,
    pub(crate) metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub(crate) struct DodopayCheckoutOutput {
    pub(crate) gateway_order_id: String,
    pub(crate) pay_amount: f64,
    pub(crate) payment_instructions: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct DodopayCreateCheckoutResponse {
    session_id: String,
    checkout_url: String,
    #[serde(default)]
    payment_id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DodopayPaymentChannel {
    pub(crate) channel: &'static str,
    pub(crate) display_name: &'static str,
}

fn normalize_base_url(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    let parsed = url::Url::parse(trimmed).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return None;
    }
    Some(trimmed.to_string())
}

fn forwarded_header_first(value: String) -> Option<String> {
    value
        .split(',')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) async fn load_dodopay_config(state: &AppState) -> Result<DodopayConfig, String> {
    let Some(record) = state
        .find_payment_gateway_config("dodopay")
        .await
        .map_err(|err| format!("dodopay config lookup failed: {err:?}"))?
    else {
        return Err("DoDoPay 未配置".to_string());
    };
    if !record.enabled {
        return Err("DoDoPay 未启用".to_string());
    }
    let Some(encrypted_secret) = record.merchant_key_encrypted.as_deref() else {
        return Err("DoDoPay API Key 未配置".to_string());
    };
    let Some(api_key) = crate::handlers::shared::decrypt_catalog_secret_with_fallbacks(
        state.encryption_key(),
        encrypted_secret,
    ) else {
        return Err("DoDoPay API Key 解密失败".to_string());
    };
    let webhook_secret = record
        .webhook_secret_encrypted
        .as_deref()
        .map(|encrypted| {
            crate::handlers::shared::decrypt_catalog_secret_with_fallbacks(
                state.encryption_key(),
                encrypted,
            )
            .ok_or_else(|| "DoDoPay Webhook Secret 解密失败".to_string())
        })
        .transpose()?;
    let Some(webhook_secret) = webhook_secret
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Err("DoDoPay Webhook Secret 未配置".to_string());
    };
    let product_id = record.merchant_id.trim();
    if product_id.is_empty() {
        return Err("DoDoPay 产品 ID 未配置".to_string());
    }
    let Some(base_url) = normalize_base_url(&record.endpoint_url) else {
        return Err("DoDoPay 服务地址必须是 http(s) 绝对地址".to_string());
    };
    let callback_base_url = record.callback_base_url;
    if let Some(value) = callback_base_url.as_deref() {
        if normalize_base_url(value).is_none() {
            return Err("DoDoPay 回调站点根地址必须是 http(s) 绝对地址".to_string());
        }
    }
    Ok(DodopayConfig {
        base_url,
        product_id: product_id.to_string(),
        api_key,
        webhook_secret: Some(webhook_secret),
        callback_base_url,
        return_path: "/dashboard/wallet".to_string(),
        pay_currency: record.pay_currency,
        usd_exchange_rate: record.usd_exchange_rate,
        min_recharge_usd: record.min_recharge_usd,
    })
}

pub(crate) fn dodopay_callback_base_url(
    configured: Option<&str>,
    headers: &http::HeaderMap,
    request_context: &GatewayPublicRequestContext,
) -> Option<String> {
    if let Some(value) = configured.and_then(normalize_base_url) {
        return Some(value);
    }

    if let Some(value) = std::env::var("AETHER_PUBLIC_BASE_URL")
        .ok()
        .or_else(|| std::env::var("PUBLIC_BASE_URL").ok())
        .and_then(|value| normalize_base_url(&value))
    {
        return Some(value);
    }

    let host = crate::headers::header_value_str(headers, crate::constants::FORWARDED_HOST_HEADER)
        .and_then(forwarded_header_first)
        .or_else(|| request_context.host_header.clone())
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| {
            !value.is_empty()
                && !value.contains('/')
                && !value.contains('\\')
                && !value.contains('@')
                && !value.contains(char::is_whitespace)
        })?;
    let proto = crate::headers::header_value_str(headers, crate::constants::FORWARDED_PROTO_HEADER)
        .and_then(forwarded_header_first)
        .map(|value| value.trim().trim_end_matches(':').to_ascii_lowercase())
        .filter(|value| value == "http" || value == "https")
        .unwrap_or_else(|| "http".to_string());
    normalize_base_url(&format!("{proto}://{host}"))
}

fn normalize_return_path(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

pub(crate) fn dodopay_return_url(config: &DodopayConfig, callback_base_url: &str) -> String {
    format!(
        "{}{}",
        callback_base_url.trim_end_matches('/'),
        normalize_return_path(&config.return_path)
    )
}

fn dodopay_cancel_token(secret: &str, order_no: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.trim().as_bytes()).expect("hmac key should work");
    mac.update(b"dodopay.cancel.");
    mac.update(order_no.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

fn dodopay_verify_cancel_token(secret: &str, order_no: &str, token: &str) -> bool {
    let expected = dodopay_cancel_token(secret, order_no);
    dodopay_timing_safe_equal(token.trim(), &expected)
}

pub(crate) fn dodopay_cancel_url(
    callback_base_url: &str,
    order_no: &str,
    signing_secret: &str,
) -> String {
    let encoded_order_no =
        url::form_urlencoded::byte_serialize(order_no.as_bytes()).collect::<String>();
    let token = dodopay_cancel_token(signing_secret, order_no);
    let encoded_token = url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>();
    format!(
        "{}/api/payment/dodopay/cancel?order_no={encoded_order_no}&token={encoded_token}",
        callback_base_url.trim_end_matches('/')
    )
}

pub(crate) fn configured_dodopay_channels() -> [DodopayPaymentChannel; 2] {
    [
        DodopayPaymentChannel {
            channel: "ali_pay",
            display_name: "支付宝",
        },
        DodopayPaymentChannel {
            channel: "we_chat_pay",
            display_name: "微信支付",
        },
    ]
}

pub(crate) fn normalize_dodopay_payment_channel(value: Option<&str>) -> Result<String, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err("请选择 DoDoPay 支付方式".to_string());
    };
    let normalized = value
        .chars()
        .filter(|ch| *ch != '-' && *ch != '_' && !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "alipay" | "ali" => Ok("ali_pay".to_string()),
        "wechatpay" | "wechat" | "weixin" | "wxpay" | "wx" => Ok("we_chat_pay".to_string()),
        _ => Err("DoDoPay 只支持选择支付宝或微信支付".to_string()),
    }
}

fn dodopay_payment_channel_display_name(channel: &str) -> &'static str {
    configured_dodopay_channels()
        .into_iter()
        .find(|item| item.channel == channel)
        .map(|item| item.display_name)
        .unwrap_or("DoDoPay")
}

fn dodopay_canonicalize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut items = map.iter().collect::<Vec<_>>();
            items.sort_by(|left, right| left.0.cmp(right.0));
            let mut object = serde_json::Map::new();
            for (key, value) in items {
                object.insert(key.clone(), dodopay_canonicalize_json(value));
            }
            serde_json::Value::Object(object)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(dodopay_canonicalize_json).collect())
        }
        _ => value.clone(),
    }
}

fn dodopay_unsigned_payload(payload: &serde_json::Value) -> serde_json::Value {
    let mut unsigned = payload.clone();
    if let serde_json::Value::Object(object) = &mut unsigned {
        object.remove("signature");
    }
    unsigned
}

pub(crate) fn dodopay_sign_payload(
    app_secret: &str,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let canonical = dodopay_canonicalize_json(&dodopay_unsigned_payload(payload));
    let encoded = serde_json::to_string(&canonical)
        .map_err(|err| format!("dodopay payload encode failed: {err}"))?;
    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes())
        .map_err(|err| format!("dodopay hmac init failed: {err}"))?;
    mac.update(encoded.as_bytes());
    let bytes = mac.finalize().into_bytes();
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn dodopay_timing_safe_equal(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .fold(0u8, |acc, (left, right)| acc | (left ^ right))
            == 0
}

pub(crate) fn dodopay_verify_payload_signature(
    app_secret: &str,
    payload: &serde_json::Value,
) -> Result<bool, String> {
    let provided = payload
        .get("signature")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("")
        .to_ascii_lowercase();
    if provided.is_empty() {
        return Ok(false);
    }
    let expected = dodopay_sign_payload(app_secret, payload)?;
    Ok(dodopay_timing_safe_equal(&provided, &expected))
}

fn dodopay_has_standard_webhook_headers(headers: &http::HeaderMap) -> bool {
    crate::headers::header_value_str(headers, "webhook-id").is_some()
        || crate::headers::header_value_str(headers, "webhook-timestamp").is_some()
        || crate::headers::header_value_str(headers, "webhook-signature").is_some()
}

fn dodopay_standard_webhook_id(headers: &http::HeaderMap) -> Option<String> {
    crate::headers::header_value_str(headers, "webhook-id")
}

fn dodopay_standard_webhook_secret_key(secret: &str) -> Vec<u8> {
    let trimmed = secret.trim();
    let Some(encoded) = trimmed.strip_prefix("whsec_") else {
        return trimmed.as_bytes().to_vec();
    };
    STANDARD
        .decode(encoded)
        .ok()
        .filter(|bytes| !bytes.is_empty())
        .or_else(|| {
            URL_SAFE_NO_PAD
                .decode(encoded)
                .ok()
                .filter(|bytes| !bytes.is_empty())
        })
        .unwrap_or_else(|| trimmed.as_bytes().to_vec())
}

fn dodopay_standard_webhook_signature_candidates(header: &str) -> Vec<String> {
    header
        .split_whitespace()
        .filter_map(|item| {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                return None;
            }
            if let Some((version, signature)) = trimmed.split_once(',') {
                if version.eq_ignore_ascii_case("v1") {
                    return Some(signature.trim().to_string());
                }
            }
            Some(trimmed.to_string())
        })
        .filter(|value| !value.is_empty())
        .collect()
}

pub(crate) fn dodopay_verify_standard_webhook_signature(
    webhook_secret: &str,
    headers: &http::HeaderMap,
    raw_body: &[u8],
) -> Result<bool, String> {
    let webhook_id = crate::headers::header_value_str(headers, "webhook-id")
        .ok_or_else(|| "missing webhook-id".to_string())?;
    let webhook_timestamp = crate::headers::header_value_str(headers, "webhook-timestamp")
        .ok_or_else(|| "missing webhook-timestamp".to_string())?;
    let webhook_signature = crate::headers::header_value_str(headers, "webhook-signature")
        .ok_or_else(|| "missing webhook-signature".to_string())?;
    let key = dodopay_standard_webhook_secret_key(webhook_secret);
    if key.is_empty() {
        return Err("empty webhook secret".to_string());
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(&key)
        .map_err(|err| format!("dodopay webhook hmac init failed: {err}"))?;
    mac.update(webhook_id.as_bytes());
    mac.update(b".");
    mac.update(webhook_timestamp.as_bytes());
    mac.update(b".");
    mac.update(raw_body);
    let expected = STANDARD.encode(mac.finalize().into_bytes());
    Ok(
        dodopay_standard_webhook_signature_candidates(&webhook_signature)
            .iter()
            .any(|candidate| dodopay_timing_safe_equal(candidate, &expected)),
    )
}

fn dodopay_checkout_url(config: &DodopayConfig) -> String {
    format!("{}/checkouts", config.base_url.trim_end_matches('/'))
}

pub(crate) async fn create_dodopay_checkout(
    config: &DodopayConfig,
    input: &DodopayCheckoutInput,
) -> Result<DodopayCheckoutOutput, String> {
    if !input.pay_amount.is_finite() || input.pay_amount <= 0.0 {
        return Err("dodopay amount is invalid".to_string());
    }
    let amount_minor_units = (input.pay_amount * 100.0).round() as i64;
    if amount_minor_units <= 0 {
        return Err("dodopay amount is invalid".to_string());
    }
    let mut metadata = match input.metadata.clone() {
        serde_json::Value::Object(map) => map,
        value if value.is_null() => serde_json::Map::new(),
        value => {
            let mut map = serde_json::Map::new();
            map.insert("raw_metadata".to_string(), value);
            map
        }
    };
    metadata.insert(
        "order_no".to_string(),
        serde_json::Value::String(input.order_no.clone()),
    );
    metadata.insert(
        "subject".to_string(),
        serde_json::Value::String(input.subject.clone()),
    );
    let body = json!({
        "product_cart": [{
            "product_id": config.product_id,
            "quantity": 1,
            "amount": amount_minor_units,
        }],
        "allowed_payment_method_types": [input.payment_channel],
        "billing_currency": config.pay_currency,
        "return_url": input.return_url,
        "cancel_url": input.cancel_url,
        "metadata": serde_json::Value::Object(metadata),
    });

    let response = reqwest::Client::new()
        .post(dodopay_checkout_url(config))
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("dodopay create checkout failed: {err}"))?;
    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|err| format!("dodopay response read failed: {err}"))?;
    if !status.is_success() {
        return Err(format!(
            "dodopay create checkout returned {status}: {response_text}"
        ));
    }
    let checkout: DodopayCreateCheckoutResponse = serde_json::from_str(&response_text)
        .map_err(|err| format!("dodopay response parse failed: {err}"))?;
    let checkout_url = checkout.checkout_url.trim().to_string();
    if checkout_url.is_empty() {
        return Err("dodopay checkout_url is empty".to_string());
    }
    let gateway_order_id = checkout
        .payment_id
        .clone()
        .unwrap_or_else(|| checkout.session_id.clone());
    let payment_instructions = json!({
        "gateway": "dodopay",
        "display_name": dodopay_payment_channel_display_name(&input.payment_channel),
        "gateway_order_id": gateway_order_id,
        "checkout_session_id": checkout.session_id,
        "payment_id": checkout.payment_id,
        "payment_url": checkout_url,
        "submit_method": "GET",
        "qr_code": serde_json::Value::Null,
        "pay_amount": input.pay_amount,
        "pay_currency": config.pay_currency,
        "payment_channel": input.payment_channel,
        "provider_order_status": "checkout_created",
        "expires_at": serde_json::Value::Null,
    });

    Ok(DodopayCheckoutOutput {
        gateway_order_id,
        pay_amount: input.pay_amount,
        payment_instructions,
    })
}

fn dodopay_plain(status: http::StatusCode, body: &'static str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(body))
        .expect("dodopay plain response should build")
}

fn dodopay_redirect(location: String) -> Response<Body> {
    Response::builder()
        .status(http::StatusCode::FOUND)
        .header(http::header::LOCATION, location)
        .body(Body::empty())
        .expect("dodopay redirect response should build")
}

fn dodopay_return_location(query: Option<&str>) -> String {
    let suffix = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("?{value}"))
        .unwrap_or_default();
    format!("/dashboard/wallet{suffix}")
}

pub(super) async fn handle_dodopay_return(
    request_context: &GatewayPublicRequestContext,
) -> Response<Body> {
    dodopay_redirect(dodopay_return_location(
        request_context.request_query_string.as_deref(),
    ))
}

fn dodopay_query_param(query: Option<&str>, key: &str) -> Option<String> {
    url::form_urlencoded::parse(query.unwrap_or_default().as_bytes())
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn dodopay_cancel_location() -> String {
    "/dashboard/wallet?payment_cancelled=1".to_string()
}

fn dodopay_value_at<'a>(
    value: &'a serde_json::Value,
    path: &[&str],
) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn dodopay_string_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.trim().to_string()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
    .filter(|value| !value.is_empty())
}

fn dodopay_string_at(value: &serde_json::Value, path: &[&str]) -> Option<String> {
    dodopay_value_at(value, path).and_then(dodopay_string_value)
}

fn dodopay_callback_object(payload: &serde_json::Value) -> &serde_json::Value {
    dodopay_value_at(payload, &["data", "object"])
        .or_else(|| dodopay_value_at(payload, &["data"]))
        .filter(|value| value.is_object())
        .unwrap_or(payload)
}

fn dodopay_callback_event_type(payload: &serde_json::Value) -> Option<String> {
    dodopay_string_at(payload, &["type"])
        .or_else(|| dodopay_string_at(payload, &["event_type"]))
        .map(|value| value.to_ascii_lowercase())
}

fn dodopay_is_success_event(payload: &serde_json::Value) -> bool {
    matches!(
        dodopay_callback_event_type(payload).as_deref(),
        Some("payment.succeeded" | "payment.success" | "payment_succeeded")
    )
}

fn dodopay_product_matches(payload: &serde_json::Value, product_id: &str) -> bool {
    let object = dodopay_callback_object(payload);
    let matches_direct_product = dodopay_string_at(payload, &["app_id"])
        .or_else(|| dodopay_string_at(object, &["app_id"]))
        .or_else(|| dodopay_string_at(payload, &["product_id"]))
        .or_else(|| dodopay_string_at(object, &["product_id"]))
        .map(|value| value == product_id);
    if let Some(matches) = matches_direct_product {
        return matches;
    }

    let carts = [
        dodopay_value_at(payload, &["product_cart"]),
        dodopay_value_at(object, &["product_cart"]),
    ];
    let mut saw_product_id = false;
    for cart in carts.into_iter().flatten() {
        let Some(items) = cart.as_array() else {
            continue;
        };
        for item in items {
            if let Some(value) = dodopay_string_at(item, &["product_id"]) {
                saw_product_id = true;
                if value == product_id {
                    return true;
                }
            }
        }
    }
    !saw_product_id
}

fn dodopay_callback_order_no(payload: &serde_json::Value) -> Option<String> {
    let object = dodopay_callback_object(payload);
    dodopay_string_at(object, &["metadata", "order_no"])
        .or_else(|| dodopay_string_at(payload, &["metadata", "order_no"]))
        .or_else(|| dodopay_string_at(object, &["merchant_order_id"]))
        .or_else(|| dodopay_string_at(payload, &["merchant_order_id"]))
}

fn dodopay_callback_gateway_order_id(payload: &serde_json::Value) -> Option<String> {
    let object = dodopay_callback_object(payload);
    dodopay_string_at(object, &["payment_id"])
        .or_else(|| dodopay_string_at(payload, &["payment_id"]))
        .or_else(|| dodopay_string_at(object, &["checkout_session_id"]))
        .or_else(|| dodopay_string_at(payload, &["checkout_session_id"]))
        .or_else(|| dodopay_string_at(object, &["order_id"]))
        .or_else(|| dodopay_string_at(payload, &["order_id"]))
        .or_else(|| dodopay_string_at(object, &["id"]))
}

fn dodopay_callback_channel(payload: &serde_json::Value) -> Option<String> {
    let object = dodopay_callback_object(payload);
    dodopay_string_at(object, &["payment_method_type"])
        .or_else(|| dodopay_string_at(object, &["payment_method"]))
        .or_else(|| dodopay_string_at(object, &["channel"]))
        .or_else(|| dodopay_string_at(payload, &["channel"]))
        .and_then(|value| normalize_dodopay_payment_channel(Some(&value)).ok())
}

fn dodopay_callback_currency(payload: &serde_json::Value) -> Option<String> {
    let object = dodopay_callback_object(payload);
    dodopay_string_at(object, &["currency"])
        .or_else(|| dodopay_string_at(payload, &["currency"]))
        .map(|value| value.to_ascii_uppercase())
}

fn dodopay_amount_value(value: &serde_json::Value, minor_units: bool) -> Option<f64> {
    let amount = match value {
        serde_json::Value::Number(number) if minor_units => number.as_f64()? / 100.0,
        serde_json::Value::Number(number) => number.as_f64()?,
        serde_json::Value::String(value) if minor_units && !value.contains('.') => {
            value.trim().parse::<f64>().ok()? / 100.0
        }
        serde_json::Value::String(value) => value.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    amount
        .is_finite()
        .then_some(amount)
        .filter(|value| *value > 0.0)
}

fn dodopay_amount_at(value: &serde_json::Value, path: &[&str], minor_units: bool) -> Option<f64> {
    dodopay_value_at(value, path).and_then(|value| dodopay_amount_value(value, minor_units))
}

fn dodopay_amount_at_allow_zero(
    value: &serde_json::Value,
    path: &[&str],
    minor_units: bool,
) -> Option<f64> {
    let amount = match dodopay_value_at(value, path)? {
        serde_json::Value::Number(number) if minor_units => number.as_f64()? / 100.0,
        serde_json::Value::Number(number) => number.as_f64()?,
        serde_json::Value::String(value) if minor_units && !value.contains('.') => {
            value.trim().parse::<f64>().ok()? / 100.0
        }
        serde_json::Value::String(value) => value.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    amount
        .is_finite()
        .then_some(amount)
        .filter(|value| *value >= 0.0)
}

fn dodopay_total_amount_excluding_tax(payload: &serde_json::Value) -> Option<f64> {
    let object = dodopay_callback_object(payload);
    let total_amount = dodopay_amount_at(object, &["total_amount"], true)
        .or_else(|| dodopay_amount_at(payload, &["total_amount"], true))?;
    let tax = dodopay_amount_at_allow_zero(object, &["tax"], true)
        .or_else(|| dodopay_amount_at_allow_zero(payload, &["tax"], true))
        .unwrap_or(0.0);
    let amount = ((total_amount - tax) * 100.0).round() / 100.0;
    amount
        .is_finite()
        .then_some(amount)
        .filter(|value| *value > 0.0)
}

fn dodopay_callback_pay_amount(payload: &serde_json::Value) -> Option<f64> {
    let object = dodopay_callback_object(payload);
    dodopay_amount_at(object, &["payment_amount"], true)
        .or_else(|| dodopay_amount_at(payload, &["payment_amount"], true))
        .or_else(|| dodopay_amount_at(object, &["payable_amount"], false))
        .or_else(|| dodopay_amount_at(payload, &["payable_amount"], false))
        .or_else(|| dodopay_total_amount_excluding_tax(payload))
        .or_else(|| dodopay_amount_at(object, &["amount"], true))
        .or_else(|| dodopay_amount_at(payload, &["amount"], true))
}

pub(super) async fn handle_dodopay_cancel(
    state: &AppState,
    request_context: &GatewayPublicRequestContext,
) -> Response<Body> {
    let query = request_context.request_query_string.as_deref();
    if let (Some(order_no), Some(token)) = (
        dodopay_query_param(query, "order_no"),
        dodopay_query_param(query, "token"),
    ) {
        match load_dodopay_config(state).await {
            Ok(config) if dodopay_verify_cancel_token(&config.api_key, &order_no, &token) => {
                let outcome = state
                    .cancel_payment_order(
                        aether_data::repository::wallet::CancelPaymentOrderInput {
                            order_no,
                            expected_payment_provider: Some("dodopay".to_string()),
                            cancel_reason: "user_cancelled_at_gateway".to_string(),
                            cancel_source: "dodopay_cancel_url".to_string(),
                        },
                    )
                    .await;
                if let Err(err) = outcome {
                    warn!(error = ?err, "failed to mark dodopay order as cancelled");
                }
            }
            Ok(_) => {
                warn!("rejected dodopay cancel callback with invalid token");
            }
            Err(err) => {
                warn!(error = %err, "failed to load dodopay config for cancel callback");
            }
        }
    }
    dodopay_redirect(dodopay_cancel_location())
}

pub(super) async fn handle_dodopay_notify(
    state: &AppState,
    headers: &http::HeaderMap,
    request_body: Option<&axum::body::Bytes>,
) -> Response<Body> {
    let config = match load_dodopay_config(state).await {
        Ok(value) => value,
        Err(_) => return dodopay_plain(http::StatusCode::SERVICE_UNAVAILABLE, "fail"),
    };
    let Some(request_body) = request_body else {
        return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail");
    };
    let payload: serde_json::Value = match serde_json::from_slice(request_body) {
        Ok(value) => value,
        Err(_) => return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail"),
    };
    let has_standard_headers = dodopay_has_standard_webhook_headers(headers);
    let signature_valid = if has_standard_headers {
        let Some(webhook_secret) = config.webhook_secret.as_deref() else {
            return dodopay_plain(http::StatusCode::SERVICE_UNAVAILABLE, "fail");
        };
        dodopay_verify_standard_webhook_signature(webhook_secret, headers, request_body)
            .unwrap_or_default()
    } else {
        dodopay_verify_payload_signature(&config.api_key, &payload).unwrap_or_default()
    };
    if !signature_valid {
        return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail");
    }
    if !dodopay_is_success_event(&payload) {
        return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail");
    }
    if !dodopay_product_matches(&payload, &config.product_id) {
        return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail");
    }
    if dodopay_callback_currency(&payload)
        .as_deref()
        .is_some_and(|currency| !currency.eq_ignore_ascii_case(&config.pay_currency))
    {
        return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail");
    }
    let order_no = dodopay_callback_order_no(&payload);
    let gateway_order_id = dodopay_callback_gateway_order_id(&payload);
    if order_no.is_none() && gateway_order_id.is_none() {
        return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail");
    }
    let Some(pay_amount) = dodopay_callback_pay_amount(&payload) else {
        return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail");
    };
    let amount_usd = if config.usd_exchange_rate > 0.0 {
        pay_amount / config.usd_exchange_rate
    } else {
        pay_amount
    };
    let channel = dodopay_callback_channel(&payload);
    let payload_hash = match payment_callback_payload_hash(&payload) {
        Ok(value) => value,
        Err(_) => return dodopay_plain(http::StatusCode::BAD_REQUEST, "fail"),
    };
    let callback_key = if has_standard_headers {
        dodopay_standard_webhook_id(headers)
    } else {
        dodopay_string_at(&payload, &["event_id"])
    }
    .unwrap_or_else(|| {
        gateway_order_id
            .as_deref()
            .or(order_no.as_deref())
            .map(|value| format!("dodopay:{value}:{payload_hash}"))
            .unwrap_or_else(|| format!("dodopay:{payload_hash}"))
    });

    let outcome = state
        .process_payment_callback(
            aether_data::repository::wallet::ProcessPaymentCallbackInput {
                payment_method: "dodopay".to_string(),
                payment_provider: Some("dodopay".to_string()),
                payment_channel: channel,
                callback_key,
                order_no,
                gateway_order_id,
                amount_usd,
                pay_amount: Some(pay_amount),
                pay_currency: Some(config.pay_currency),
                exchange_rate: Some(config.usd_exchange_rate),
                payload_hash,
                payload,
                signature_valid: true,
            },
        )
        .await;

    match outcome {
        Ok(Some(aether_data::repository::wallet::ProcessPaymentCallbackOutcome::Applied {
            order,
            order_id,
            ..
        })) => {
            if let Err(err) = state.apply_referral_rewards_for_paid_order(&order).await {
                warn!(
                    error = ?err,
                    order_id = %order_id,
                    "failed to apply referral rewards for dodopay callback"
                );
            }
            dodopay_plain(http::StatusCode::OK, "success")
        }
        Ok(Some(
            aether_data::repository::wallet::ProcessPaymentCallbackOutcome::AlreadyCredited {
                ..
            },
        ))
        | Ok(Some(
            aether_data::repository::wallet::ProcessPaymentCallbackOutcome::DuplicateProcessed {
                ..
            },
        )) => dodopay_plain(http::StatusCode::OK, "success"),
        _ => dodopay_plain(http::StatusCode::INTERNAL_SERVER_ERROR, "fail"),
    }
}

#[cfg(test)]
mod tests {
    use aether_data::repository::wallet::{
        CreateWalletRechargeOrderInput, CreateWalletRechargeOrderOutcome, StoredAdminPaymentOrder,
    };
    use aether_data_contracts::repository::billing::PaymentGatewayConfigWriteInput;
    use base64::Engine as _;
    use hmac::Mac;
    use serde_json::json;

    async fn dodopay_test_state_with_webhook(
        api_key: &str,
        webhook_secret: Option<&str>,
    ) -> crate::AppState {
        let auth_repository = std::sync::Arc::new(
            aether_data::repository::auth::InMemoryAuthApiKeySnapshotRepository::default(),
        );
        let billing_repository: std::sync::Arc<
            dyn aether_data_contracts::repository::billing::BillingReadRepository,
        > = std::sync::Arc::new(
            aether_data::repository::billing::InMemoryBillingReadRepository::default(),
        );
        let wallet_repository = std::sync::Arc::new(
            aether_data::repository::wallet::InMemoryWalletRepository::default(),
        );
        let state = crate::AppState::new()
            .expect("state should build")
            .with_data_state_for_tests(
                crate::data::GatewayDataState::with_auth_billing_and_wallet_for_tests(
                    auth_repository,
                    billing_repository,
                    wallet_repository,
                ),
            );
        let merchant_key_encrypted =
            crate::handlers::shared::encrypt_catalog_secret_with_fallbacks(&state, api_key)
                .expect("api key should encrypt");
        let webhook_secret_encrypted = webhook_secret.map(|secret| {
            crate::handlers::shared::encrypt_catalog_secret_with_fallbacks(&state, secret)
                .expect("webhook secret should encrypt")
        });
        let preserve_existing_webhook_secret = webhook_secret_encrypted.is_none();
        let outcome = state
            .upsert_payment_gateway_config(&PaymentGatewayConfigWriteInput {
                provider: "dodopay".to_string(),
                enabled: true,
                endpoint_url: "https://test.dodopayments.com".to_string(),
                callback_base_url: Some("https://aether.example.com".to_string()),
                merchant_id: "product_123".to_string(),
                merchant_key_encrypted: Some(merchant_key_encrypted),
                preserve_existing_secret: false,
                webhook_secret_encrypted,
                preserve_existing_webhook_secret,
                pay_currency: "USD".to_string(),
                usd_exchange_rate: 1.0,
                min_recharge_usd: 1.0,
                channels_json: json!({}),
            })
            .await
            .expect("gateway config should save");
        assert!(matches!(outcome, crate::LocalMutationOutcome::Applied(_)));
        state
    }

    async fn dodopay_test_state(api_key: &str) -> crate::AppState {
        dodopay_test_state_with_webhook(api_key, Some("dodopay-webhook-secret")).await
    }

    async fn create_cancel_test_order(
        state: &crate::AppState,
        order_no: &str,
        provider: &str,
    ) -> StoredAdminPaymentOrder {
        let outcome = state
            .create_wallet_recharge_order(CreateWalletRechargeOrderInput {
                preferred_wallet_id: Some(format!("wallet-{order_no}")),
                user_id: format!("user-{order_no}"),
                amount_usd: 3.0,
                pay_amount: Some(3.0),
                pay_currency: Some("USD".to_string()),
                exchange_rate: Some(1.0),
                payment_method: provider.to_string(),
                payment_provider: Some(provider.to_string()),
                payment_channel: Some("ali_pay".to_string()),
                gateway_order_id: format!("gateway-{order_no}"),
                gateway_response: json!({ "gateway": provider }),
                order_no: order_no.to_string(),
                expires_at_unix_secs: 4_102_444_800,
            })
            .await
            .expect("order create should run")
            .expect("wallet writer should exist");
        match outcome {
            CreateWalletRechargeOrderOutcome::Created(order) => order,
            CreateWalletRechargeOrderOutcome::WalletInactive => {
                panic!("new test wallet should be active")
            }
        }
    }

    fn cancel_request_context(
        query: impl Into<String>,
    ) -> crate::control::GatewayPublicRequestContext {
        crate::control::GatewayPublicRequestContext {
            trace_id: "trace-dodopay-cancel-test".to_string(),
            request_method: axum::http::Method::GET,
            request_path: "/api/payment/dodopay/cancel".to_string(),
            request_query_string: Some(query.into()),
            request_content_type: None,
            host_header: Some("aether.example.com".to_string()),
            control_decision: None,
        }
    }

    async fn read_cancel_test_order(
        state: &crate::AppState,
        user_id: &str,
        order_id: String,
    ) -> StoredAdminPaymentOrder {
        state
            .find_wallet_payment_order_by_user_id(user_id, &order_id)
            .await
            .expect("order read should run")
            .expect("order should exist")
    }

    fn dodopay_standard_webhook_headers(
        secret: &[u8],
        webhook_id: &str,
        timestamp: &str,
        raw_body: &[u8],
    ) -> axum::http::HeaderMap {
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(secret).expect("hmac");
        mac.update(webhook_id.as_bytes());
        mac.update(b".");
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(raw_body);
        let signature =
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "webhook-id",
            axum::http::HeaderValue::from_str(webhook_id).expect("header"),
        );
        headers.insert(
            "webhook-timestamp",
            axum::http::HeaderValue::from_str(timestamp).expect("header"),
        );
        headers.insert(
            "webhook-signature",
            axum::http::HeaderValue::from_str(&format!("v1,wrong v1,{signature}")).expect("header"),
        );
        headers
    }

    #[test]
    fn dodopay_signs_stable_json_without_signature() {
        let mut payload = json!({
            "timestamp": 1710000000,
            "nonce": "nonce-123456",
            "app_id": "app_test",
            "merchant_order_id": "po_test",
            "amount": "9.90",
            "subject": "钱包充值",
            "metadata": {
                "signature": "kept"
            }
        });
        let first = super::dodopay_sign_payload("secret", &payload).expect("sign should work");
        let second = super::dodopay_sign_payload("secret", &payload).expect("sign should work");
        assert_eq!(first, second);

        payload["signature"] = json!("ignored");
        let with_signature =
            super::dodopay_sign_payload("secret", &payload).expect("sign should work");
        assert_eq!(first, with_signature);

        payload["metadata"]["signature"] = json!("still-signed");
        let with_nested_signature_changed =
            super::dodopay_sign_payload("secret", &payload).expect("sign should work");
        assert_ne!(first, with_nested_signature_changed);
    }

    #[test]
    fn dodopay_callback_signature_requires_matching_secret() {
        let mut payload = json!({
            "event_id": "evt_1",
            "event_type": "payment.succeeded",
            "app_id": "app_test",
            "order_id": "order_1",
            "merchant_order_id": "po_test",
            "amount": "9.90",
            "payable_amount": "9.91",
            "channel": "ALIPAY",
            "paid_at": "2026-05-26T10:08:00.000Z",
            "metadata": null,
            "timestamp": 1710000000
        });
        let signature =
            super::dodopay_sign_payload("secret", &payload).expect("signature should build");
        payload["signature"] = json!(signature);

        assert!(super::dodopay_verify_payload_signature("secret", &payload)
            .expect("verification should work"));
        assert!(!super::dodopay_verify_payload_signature("wrong", &payload)
            .expect("verification should work"));
    }

    #[tokio::test]
    async fn dodopay_config_requires_webhook_secret() {
        let state = dodopay_test_state_with_webhook("dodopay-api-key", None).await;

        let error = super::load_dodopay_config(&state)
            .await
            .expect_err("webhook secret should be required");

        assert_eq!(error, "DoDoPay Webhook Secret 未配置");
    }

    #[test]
    fn dodopay_standard_webhook_signature_uses_raw_body_and_whsec_secret() {
        let raw_body = br#"{"type":"payment.succeeded","data":{"object":{"id":"pay_1"}}}"#;
        let changed_body = br#"{"data":{"object":{"id":"pay_1"}},"type":"payment.succeeded"}"#;
        let webhook_secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode(b"secret")
        );
        let headers = dodopay_standard_webhook_headers(b"secret", "evt_1", "1710000000", raw_body);

        assert!(super::dodopay_verify_standard_webhook_signature(
            &webhook_secret,
            &headers,
            raw_body
        )
        .expect("verification should work"));
        assert!(
            super::dodopay_verify_standard_webhook_signature("secret", &headers, raw_body)
                .expect("verification should work")
        );
        assert!(!super::dodopay_verify_standard_webhook_signature(
            &webhook_secret,
            &headers,
            changed_body
        )
        .expect("verification should work"));
    }

    #[test]
    fn dodopay_payment_channel_accepts_supported_aliases() {
        assert_eq!(
            super::normalize_dodopay_payment_channel(Some("ali_pay")).expect("channel"),
            "ali_pay"
        );
        assert_eq!(
            super::normalize_dodopay_payment_channel(Some("ALIPAY")).expect("channel"),
            "ali_pay"
        );
        assert_eq!(
            super::normalize_dodopay_payment_channel(Some("we_chat_pay")).expect("channel"),
            "we_chat_pay"
        );
        assert_eq!(
            super::normalize_dodopay_payment_channel(Some("wxpay")).expect("channel"),
            "we_chat_pay"
        );
        assert!(super::normalize_dodopay_payment_channel(Some("card")).is_err());
    }

    #[test]
    fn dodopay_cancel_url_points_to_local_cancel_route() {
        let cancel_url =
            super::dodopay_cancel_url("https://aether.example.com/", "po_1 2", "secret");

        assert!(cancel_url.starts_with(
            "https://aether.example.com/api/payment/dodopay/cancel?order_no=po_1+2&token="
        ));
        assert!(super::dodopay_verify_cancel_token(
            "secret",
            "po_1 2",
            cancel_url
                .split_once("token=")
                .map(|(_, token)| token)
                .expect("token should exist")
        ));
    }

    #[test]
    fn dodopay_cancel_token_is_bound_to_order_no() {
        let token = super::dodopay_cancel_token("secret", "po_1");

        assert!(super::dodopay_verify_cancel_token("secret", "po_1", &token));
        assert!(!super::dodopay_verify_cancel_token(
            "secret", "po_2", &token
        ));
    }

    #[tokio::test]
    async fn dodopay_cancel_callback_requires_valid_token_and_dodopay_provider() {
        let api_key = "dodopay-api-key";
        let state = dodopay_test_state(api_key).await;

        let signed_order = create_cancel_test_order(&state, "po-cancel-signed", "dodopay").await;
        let signed_url = super::dodopay_cancel_url(
            "https://aether.example.com",
            &signed_order.order_no,
            api_key,
        );
        let signed_query = signed_url
            .split_once('?')
            .map(|(_, query)| query)
            .expect("signed cancel url should contain query");
        let response =
            super::handle_dodopay_cancel(&state, &cancel_request_context(signed_query)).await;
        assert_eq!(response.status(), axum::http::StatusCode::FOUND);
        let signed_after_cancel =
            read_cancel_test_order(&state, "user-po-cancel-signed", signed_order.id).await;
        assert_eq!(signed_after_cancel.status, "cancelled");

        let unsigned_order =
            create_cancel_test_order(&state, "po-cancel-unsigned", "dodopay").await;
        let response = super::handle_dodopay_cancel(
            &state,
            &cancel_request_context(format!("order_no={}", unsigned_order.order_no)),
        )
        .await;
        assert_eq!(response.status(), axum::http::StatusCode::FOUND);
        let unsigned_after_cancel =
            read_cancel_test_order(&state, "user-po-cancel-unsigned", unsigned_order.id).await;
        assert_eq!(unsigned_after_cancel.status, "pending");

        let epay_order = create_cancel_test_order(&state, "po-cancel-epay", "epay").await;
        let epay_signed_url =
            super::dodopay_cancel_url("https://aether.example.com", &epay_order.order_no, api_key);
        let epay_signed_query = epay_signed_url
            .split_once('?')
            .map(|(_, query)| query)
            .expect("signed cancel url should contain query");
        let response =
            super::handle_dodopay_cancel(&state, &cancel_request_context(epay_signed_query)).await;
        assert_eq!(response.status(), axum::http::StatusCode::FOUND);
        let epay_after_cancel =
            read_cancel_test_order(&state, "user-po-cancel-epay", epay_order.id).await;
        assert_eq!(epay_after_cancel.status, "pending");
    }

    #[test]
    fn dodopay_callback_gateway_order_id_reads_checkout_session_id() {
        let payload = json!({
            "type": "payment.succeeded",
            "data": {
                "object": {
                    "checkout_session_id": "cs_123"
                }
            }
        });

        assert_eq!(
            super::dodopay_callback_gateway_order_id(&payload).as_deref(),
            Some("cs_123")
        );
    }

    #[test]
    fn dodopay_product_matches_official_product_cart_payload() {
        let payload = json!({
            "type": "payment.succeeded",
            "data": {
                "payment_id": "pay_123",
                "product_cart": [
                    {
                        "product_id": "pdt_123",
                        "quantity": 1
                    }
                ]
            }
        });

        assert!(super::dodopay_product_matches(&payload, "pdt_123"));
        assert!(!super::dodopay_product_matches(&payload, "pdt_other"));
    }

    #[test]
    fn dodopay_callback_currency_reads_official_payload() {
        let payload = json!({
            "type": "payment.succeeded",
            "data": {
                "payment_id": "pay_123",
                "currency": "cny"
            }
        });

        assert_eq!(
            super::dodopay_callback_currency(&payload).as_deref(),
            Some("CNY")
        );
    }

    #[test]
    fn dodopay_callback_pay_amount_excludes_reported_tax_from_total_amount() {
        let payload = json!({
            "type": "payment.succeeded",
            "data": {
                "object": {
                    "total_amount": 1099,
                    "tax": 100
                }
            }
        });

        assert_eq!(super::dodopay_callback_pay_amount(&payload), Some(9.99));
    }

    #[tokio::test]
    async fn dodopay_notify_is_disabled_without_gateway_config() {
        let state = super::AppState::new().expect("state should build");
        let body = axum::body::Bytes::from(
            serde_json::to_vec(&json!({
                "event_id": "evt_1",
                "event_type": "payment.succeeded",
                "app_id": "app_test",
                "order_id": "order_1",
                "merchant_order_id": "po_test",
                "amount": "9.90",
                "payable_amount": "9.91",
                "channel": "ALIPAY",
                "paid_at": "2026-05-26T10:08:00.000Z",
                "metadata": null,
                "timestamp": 1710000000
            }))
            .expect("payload should encode"),
        );

        let response =
            super::handle_dodopay_notify(&state, &axum::http::HeaderMap::new(), Some(&body)).await;

        assert_eq!(
            response.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }
}

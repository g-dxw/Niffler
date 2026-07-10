#[cfg(test)]
#[path = "codex/tests.rs"]
mod tests;

pub(crate) use crate::ai_serving::{
    apply_codex_openai_responses_special_body_edits,
    apply_codex_openai_responses_special_body_edits_with_bridge_config,
    apply_codex_openai_responses_special_body_edits_with_bridge_model,
    apply_codex_openai_responses_special_headers,
};

const OPENAI_RESPONSES_IMAGE_GENERATION_TOOL_ENABLED_CONFIG_KEY: &str =
    "openai_responses_image_generation_tool_enabled";
const OPENAI_INTERNAL_CODEX_RESPONSES_LITE_HEADER: &str = "x-openai-internal-codex-responses-lite";

pub(crate) fn codex_openai_image_bridge_model_from_provider_config(
    provider_config: Option<&serde_json::Value>,
) -> Option<&str> {
    provider_config
        .and_then(serde_json::Value::as_object)
        .and_then(|config| config.get("codex_image_generation_base_model"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn openai_responses_image_generation_tool_enabled_from_transport_config(
    provider_type: &str,
    provider_config: Option<&serde_json::Value>,
    endpoint_config: Option<&serde_json::Value>,
) -> bool {
    if let Some(enabled) =
        openai_responses_image_generation_tool_enabled_config_value(endpoint_config).or_else(|| {
            openai_responses_image_generation_tool_enabled_config_value(provider_config)
        })
    {
        return enabled;
    }

    matches!(
        provider_type.trim().to_ascii_lowercase().as_str(),
        "codex" | "chatgpt_web"
    )
}

pub(crate) fn openai_internal_codex_responses_lite_requested(headers: &http::HeaderMap) -> bool {
    headers
        .get(OPENAI_INTERNAL_CODEX_RESPONSES_LITE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .is_some_and(|value| value.eq_ignore_ascii_case("true") || value == "1")
}

fn openai_responses_image_generation_tool_enabled_config_value(
    config: Option<&serde_json::Value>,
) -> Option<bool> {
    config
        .and_then(serde_json::Value::as_object)
        .and_then(|config| config.get(OPENAI_RESPONSES_IMAGE_GENERATION_TOOL_ENABLED_CONFIG_KEY))
        .and_then(serde_json::Value::as_bool)
}

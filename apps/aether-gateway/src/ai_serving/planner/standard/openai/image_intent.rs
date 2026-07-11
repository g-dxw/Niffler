pub(crate) fn openai_request_is_image_generation_intent(
    requested_model: &str,
    body_json: &serde_json::Value,
) -> bool {
    openai_model_is_image_generation(requested_model)
        || body_json
            .get("model")
            .and_then(serde_json::Value::as_str)
            .is_some_and(openai_model_is_image_generation)
        || openai_tool_choice_selects_image_generation(body_json.get("tool_choice"))
}

pub(crate) fn openai_image_generation_candidate_model(
    requested_model: &str,
    body_json: &serde_json::Value,
) -> String {
    body_json
        .get("model")
        .and_then(serde_json::Value::as_str)
        .filter(|model| openai_model_is_image_generation(model))
        .or_else(|| {
            body_json
                .get("tools")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .find(|tool| {
                    tool.get("type").and_then(serde_json::Value::as_str) == Some("image_generation")
                })
                .and_then(|tool| tool.get("model"))
                .and_then(serde_json::Value::as_str)
                .filter(|model| !model.trim().is_empty())
        })
        .or_else(|| openai_model_is_image_generation(requested_model).then_some(requested_model))
        .unwrap_or(crate::ai_serving::CODEX_OPENAI_IMAGE_DEFAULT_MODEL)
        .trim()
        .to_string()
}

fn openai_model_is_image_generation(model: &str) -> bool {
    model.trim().to_ascii_lowercase().starts_with("gpt-image-")
}

fn openai_tool_choice_selects_image_generation(choice: Option<&serde_json::Value>) -> bool {
    let Some(choice) = choice else {
        return false;
    };
    if let Some(value) = choice.as_str() {
        return value.trim().eq_ignore_ascii_case("image_generation");
    }
    let Some(object) = choice.as_object() else {
        return false;
    };
    object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("image_generation"))
        || object
            .get("tool")
            .and_then(|value| value.get("type"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("image_generation"))
        || object
            .get("function")
            .and_then(|value| value.get("name"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("image_generation"))
}

#[cfg(test)]
mod tests {
    use super::{
        openai_image_generation_candidate_model, openai_request_is_image_generation_intent,
    };
    use serde_json::json;

    #[test]
    fn detects_openai_image_generation_intent_like_compat_proxies() {
        assert!(openai_request_is_image_generation_intent(
            "GPT-IMAGE-2",
            &json!({})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"model":"gpt-image-2"})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"tool_choice":{"function":{"name":"image_generation"}}})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"tool_choice":{"type":"image_generation"}})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"tools":[{"type":"image_generation"}]})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"messages":[{"role":"user","content":"hello"}]})
        ));
    }

    #[test]
    fn natural_language_image_requests_are_left_for_model_tool_selection() {
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({
                "input": [
                    {"role":"user","content":[{"type":"input_text","text":"先解释这个项目"}]},
                    {"role":"assistant","content":[{"type":"output_text","text":"好的"}]},
                    {"role":"user","content":[{"type":"input_text","text":"帮我生成一张鸭子游泳的图片"}]}
                ]
            })
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.6-sol",
            &json!({"input":"Create an image of a yellow duck swimming in a pool"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"我想生成一张图片"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"请帮我生成一张图片？"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"根据附件生成一张海报"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"Can you please create an image of a yellow duck?"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({
                "input":[{
                    "role":"user",
                    "content":[
                        {"type":"input_text","text":"根据附件"},
                        {"type":"input_text","text":"生成一张海报"}
                    ]
                }]
            })
        ));
    }

    #[test]
    fn ignores_historical_or_explanatory_image_text() {
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({
                "input": [
                    {"role":"user","content":[{"type":"input_text","text":"生成一张鸭子图片"}]},
                    {"role":"assistant","content":[{"type":"output_text","text":"已完成"}]},
                    {"role":"user","content":[{"type":"input_text","text":"现在解释这段代码"}]}
                ]
            })
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"解释一下如何调用 API 生成图片"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"生成图片的 API 应该怎么写"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"Create an image generation API in Rust"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"生成图片需要哪些参数？"})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"生成图片的参数有哪些"})
        ));
    }

    #[test]
    fn text_model_image_intent_uses_default_image_candidate_model() {
        assert_eq!(
            openai_image_generation_candidate_model(
                "gpt-5.5",
                &json!({"input":"生成一张鸭子图片"})
            ),
            "gpt-image-2"
        );
    }
}

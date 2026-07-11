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
        || latest_user_text(body_json)
            .as_deref()
            .is_some_and(explicit_image_generation_command)
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

fn latest_user_text(body_json: &serde_json::Value) -> Option<String> {
    if let Some(text) = body_json.get("input").and_then(serde_json::Value::as_str) {
        return non_empty_text(text).map(ToOwned::to_owned);
    }
    latest_role_text(body_json.get("input")).or_else(|| latest_role_text(body_json.get("messages")))
}

fn latest_role_text(value: Option<&serde_json::Value>) -> Option<String> {
    value?
        .as_array()?
        .iter()
        .rev()
        .filter_map(serde_json::Value::as_object)
        .find(|item| {
            item.get("role")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|role| role.eq_ignore_ascii_case("user"))
        })
        .and_then(|item| first_text(item.get("content")))
}

fn first_text(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::String(text) => non_empty_text(text).map(ToOwned::to_owned),
        serde_json::Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(|item| {
                    item.as_object()
                        .and_then(|object| object.get("text"))
                        .and_then(serde_json::Value::as_str)
                        .and_then(non_empty_text)
                })
                .collect::<Vec<_>>()
                .join("\n");
            (!text.is_empty()).then_some(text)
        }
        serde_json::Value::Object(object) => object
            .get("text")
            .and_then(serde_json::Value::as_str)
            .and_then(non_empty_text)
            .map(ToOwned::to_owned),
        _ => None,
    }
}

fn non_empty_text(text: &str) -> Option<&str> {
    (!text.trim().is_empty()).then_some(text)
}

fn explicit_image_generation_command(text: &str) -> bool {
    let trimmed = text.trim();
    let normalized = trimmed.to_ascii_lowercase();
    let describes_image_software = [
        "生成图片的 api",
        "生成图像的 api",
        "图片生成 api",
        "图像生成 api",
        "image generation api",
        "image generation function",
        "image generation tool",
        "image generation script",
        "image generation code",
    ]
    .iter()
    .any(|value| normalized.contains(value));
    if describes_image_software {
        return false;
    }
    if [
        "需要哪些参数",
        "需要什么参数",
        "参数有哪些",
        "参数是什么",
        "怎么生成",
        "如何生成",
        "怎么调用",
        "如何调用",
        "怎么实现",
        "如何实现",
        "怎么写",
    ]
    .iter()
    .any(|value| trimmed.contains(value))
    {
        return false;
    }

    let chinese_subject = [
        "图片", "图像", "照片", "自拍", "海报", "插画", "壁纸", "头像", "图",
    ]
    .iter()
    .any(|value| trimmed.contains(value));
    let chinese_command = strip_leading_prefixes(
        trimmed,
        &[
            "请帮我",
            "麻烦帮我",
            "帮我",
            "请",
            "麻烦",
            "给我",
            "我想",
            "我需要",
            "根据附件",
            "请根据附件",
            "根据这张图",
            "用附件",
        ],
    );
    if chinese_subject
        && [
            "生成",
            "画",
            "绘制",
            "制作",
            "创建",
            "做一张",
            "做一幅",
            "出一张",
            "出图",
        ]
        .iter()
        .any(|verb| chinese_command.starts_with(verb))
    {
        return true;
    }

    let english_command = strip_leading_prefixes(
        normalized.as_str(),
        &[
            "please ",
            "can you ",
            "could you ",
            "i want to ",
            "i need you to ",
        ],
    );
    let english_subject = [
        "image",
        "picture",
        "photo",
        "illustration",
        "poster",
        "logo",
        "icon",
        "avatar",
        "wallpaper",
    ]
    .iter()
    .any(|value| english_command.contains(value));
    english_subject
        && ["generate", "create", "draw", "render", "make", "produce"]
            .iter()
            .any(|verb| english_command.starts_with(verb))
}

fn strip_leading_prefixes<'a>(mut text: &'a str, prefixes: &[&str]) -> &'a str {
    loop {
        let trimmed = text.trim_start();
        let Some(stripped) = prefixes
            .iter()
            .find_map(|prefix| trimmed.strip_prefix(prefix))
        else {
            return trimmed;
        };
        text = stripped;
    }
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
    fn detects_explicit_image_request_from_latest_user_turn() {
        assert!(openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({
                "input": [
                    {"role":"user","content":[{"type":"input_text","text":"先解释这个项目"}]},
                    {"role":"assistant","content":[{"type":"output_text","text":"好的"}]},
                    {"role":"user","content":[{"type":"input_text","text":"帮我生成一张鸭子游泳的图片"}]}
                ]
            })
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5.6-sol",
            &json!({"input":"Create an image of a yellow duck swimming in a pool"})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"我想生成一张图片"})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"请帮我生成一张图片？"})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"根据附件生成一张海报"})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5.5",
            &json!({"input":"Can you please create an image of a yellow duck?"})
        ));
        assert!(openai_request_is_image_generation_intent(
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

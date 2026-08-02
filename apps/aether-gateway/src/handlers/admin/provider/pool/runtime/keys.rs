use base64::Engine as _;

pub(super) fn pool_sticky_pattern(provider_id: &str) -> String {
    format!("ap:{provider_id}:sticky:*")
}

pub(super) fn pool_sticky_key(provider_id: &str, session_token: &str) -> String {
    format!("ap:{provider_id}:sticky:{session_token}")
}

pub(super) fn pool_lru_key(provider_id: &str) -> String {
    format!("ap:{provider_id}:lru")
}

pub(super) fn pool_cooldown_key(provider_id: &str, key_id: &str) -> String {
    format!("ap:{provider_id}:cooldown:{key_id}")
}

pub(super) fn pool_cooldown_index_key(provider_id: &str) -> String {
    format!("ap:{provider_id}:cooldown_idx")
}

pub(super) fn pool_model_cooldown_key(provider_id: &str, key_id: &str, model_name: &str) -> String {
    let normalized_model = model_name.trim().to_ascii_lowercase();
    let encoded_model =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(normalized_model.as_bytes());
    format!("ap:{provider_id}:model_cooldown:{key_id}:{encoded_model}")
}

pub(super) fn pool_model_cooldown_index_key(provider_id: &str, key_id: &str) -> String {
    format!("ap:{provider_id}:model_cooldown_idx:{key_id}")
}

pub(super) fn pool_cost_key(provider_id: &str, key_id: &str) -> String {
    format!("ap:{provider_id}:cost:{key_id}")
}

pub(super) fn pool_latency_key(provider_id: &str, key_id: &str) -> String {
    format!("ap:{provider_id}:latency:{key_id}")
}

pub(super) fn pool_stream_timeout_key(provider_id: &str, key_id: &str) -> String {
    format!("ap:{provider_id}:stream_timeout:{key_id}")
}

pub(super) fn parse_pool_cost_member(member: &str) -> u64 {
    member
        .rsplit_once(':')
        .and_then(|(_, suffix)| suffix.parse::<u64>().ok())
        .unwrap_or(0)
}

pub(super) fn parse_pool_latency_member(member: &str) -> u64 {
    member
        .rsplit_once(':')
        .and_then(|(_, suffix)| suffix.parse::<u64>().ok())
        .unwrap_or(0)
}

pub(super) fn pool_cooldown_keys(provider_id: &str, key_ids: &[String]) -> Vec<String> {
    key_ids
        .iter()
        .map(|key_id| pool_cooldown_key(provider_id, key_id))
        .collect()
}

pub(super) fn pool_cost_keys(provider_id: &str, key_ids: &[String]) -> Vec<String> {
    key_ids
        .iter()
        .map(|key_id| pool_cost_key(provider_id, key_id))
        .collect()
}

pub(super) fn pool_latency_keys(provider_id: &str, key_ids: &[String]) -> Vec<String> {
    key_ids
        .iter()
        .map(|key_id| pool_latency_key(provider_id, key_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::pool_model_cooldown_key;

    #[test]
    fn model_cooldown_key_normalizes_model_case_and_whitespace() {
        assert_eq!(
            pool_model_cooldown_key("provider", "key", " GPT-5.6-SOL "),
            pool_model_cooldown_key("provider", "key", "gpt-5.6-sol")
        );
        assert_ne!(
            pool_model_cooldown_key("provider", "key", "gpt-5.6-sol"),
            pool_model_cooldown_key("provider", "key", "gpt-5.6-terra")
        );
    }
}

use std::collections::BTreeSet;
use std::time::Duration;

use aether_model_fetch::{
    codex_model_fetch_client_version_override, is_valid_codex_model_fetch_client_version,
    CODEX_MODEL_FETCH_CLIENT_VERSION_DEFAULT,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppState;

const CODEX_CLIENT_VERSION_STATE_KEY: &str = "codex_model_fetch_client_version_state";
const CODEX_CLIENT_VERSION_CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
pub(super) const CODEX_CLIENT_VERSION_RETRY_INTERVAL_SECS: u64 = 15 * 60;
const CODEX_NPM_LATEST_URL: &str = "https://registry.npmjs.org/@openai%2Fcodex/latest";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct CodexClientVersionState {
    pub version: String,
    pub checked_at_unix_secs: u64,
    #[serde(default)]
    pub next_check_at_unix_secs: u64,
}

pub(super) fn client_version_check_is_due(
    state: Option<&CodexClientVersionState>,
    now_unix_secs: u64,
) -> bool {
    state.is_none_or(|state| {
        let next_check_at_unix_secs = if state.next_check_at_unix_secs == 0 {
            state
                .checked_at_unix_secs
                .saturating_add(CODEX_CLIENT_VERSION_CHECK_INTERVAL_SECS)
        } else {
            state.next_check_at_unix_secs
        };
        now_unix_secs >= next_check_at_unix_secs
    })
}

pub(super) fn client_version_next_check_at(now_unix_secs: u64, check_succeeded: bool) -> u64 {
    now_unix_secs.saturating_add(if check_succeeded {
        CODEX_CLIENT_VERSION_CHECK_INTERVAL_SECS
    } else {
        CODEX_CLIENT_VERSION_RETRY_INTERVAL_SECS
    })
}

pub(super) fn parse_official_codex_stable_version(payload: &Value) -> Option<String> {
    payload
        .get("version")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|version| is_valid_codex_model_fetch_client_version(version))
        .filter(|version| Version::parse(version).is_ok_and(|value| value.pre.is_empty()))
        .map(ToOwned::to_owned)
}

pub(super) fn discovered_version_is_newer(current: &str, discovered: &str) -> bool {
    let Ok(current) = Version::parse(current.trim()) else {
        return true;
    };
    let Ok(discovered) = Version::parse(discovered.trim()) else {
        return false;
    };
    discovered.pre.is_empty() && discovered > current
}

pub(super) fn effective_version_after_probe(
    current: &str,
    discovered: &str,
    probe_succeeded: bool,
) -> String {
    if probe_succeeded && discovered_version_is_newer(current, discovered) {
        discovered.to_string()
    } else {
        current.to_string()
    }
}

pub(super) fn candidate_catalog_preserves_current_models(
    current_model_ids: &[String],
    candidate_model_ids: &[String],
) -> bool {
    if current_model_ids.is_empty() || candidate_model_ids.is_empty() {
        return false;
    }
    let current = current_model_ids
        .iter()
        .map(|model| model.trim())
        .filter(|model| !model.is_empty())
        .collect::<BTreeSet<_>>();
    let candidate = candidate_model_ids
        .iter()
        .map(|model| model.trim())
        .filter(|model| !model.is_empty())
        .collect::<BTreeSet<_>>();
    !current.is_empty() && current.is_subset(&candidate)
}

pub(super) fn effective_version_from_state_and_memory(
    persisted: Option<&CodexClientVersionState>,
    process_last_known: Option<&str>,
) -> String {
    persisted
        .map(|state| state.version.clone())
        .or_else(|| process_last_known.map(ToOwned::to_owned))
        .unwrap_or_else(|| CODEX_MODEL_FETCH_CLIENT_VERSION_DEFAULT.to_string())
}

pub(super) fn process_last_known_version(state: &AppState) -> Option<String> {
    state
        .codex_model_fetch_client_version_cache
        .lock()
        .expect("Codex model fetch client version cache should lock")
        .clone()
}

fn remember_process_last_known_version(state: &AppState, version: &str) {
    *state
        .codex_model_fetch_client_version_cache
        .lock()
        .expect("Codex model fetch client version cache should lock") = Some(version.to_string());
}

pub(super) async fn read_codex_client_version_state(
    state: &AppState,
) -> Result<Option<CodexClientVersionState>, String> {
    let Some(raw) = state
        .read_system_config_json_value(CODEX_CLIENT_VERSION_STATE_KEY)
        .await
        .map_err(|err| format!("read Codex client version state failed: {err:?}"))?
    else {
        return Ok(None);
    };
    let value = parse_codex_client_version_state(raw)?;
    remember_process_last_known_version(state, &value.version);
    Ok(Some(value))
}

fn parse_codex_client_version_state(raw: Value) -> Result<CodexClientVersionState, String> {
    let value = serde_json::from_value::<CodexClientVersionState>(raw)
        .map_err(|err| format!("parse Codex client version state failed: {err}"))?;
    if !is_valid_codex_model_fetch_client_version(&value.version) {
        return Err("stored Codex client version is invalid".to_string());
    }
    Ok(value)
}

pub(super) async fn write_codex_client_version_state(
    state: &AppState,
    value: &CodexClientVersionState,
) -> Result<(), String> {
    let serialized = serde_json::to_value(value)
        .map_err(|err| format!("serialize Codex client version state failed: {err}"))?;
    state
        .upsert_system_config_json_value(
            CODEX_CLIENT_VERSION_STATE_KEY,
            &serialized,
            Some("Codex 模型目录已验证客户端版本"),
        )
        .await
        .map_err(|err| format!("store Codex client version state failed: {err:?}"))?;
    remember_process_last_known_version(state, &value.version);
    Ok(())
}

pub(crate) async fn resolve_effective_codex_model_fetch_client_version(state: &AppState) -> String {
    if let Some(version) = codex_model_fetch_client_version_override() {
        return version;
    }
    let process_last_known = process_last_known_version(state);
    let persisted = read_codex_client_version_state(state).await.ok().flatten();
    effective_version_from_state_and_memory(persisted.as_ref(), process_last_known.as_deref())
}

pub(super) async fn fetch_official_codex_stable_version(
    state: &AppState,
) -> Result<String, String> {
    let response = state
        .client
        .get(CODEX_NPM_LATEST_URL)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::USER_AGENT, "niffler-model-fetch/1.0")
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|err| format!("fetch official Codex version failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "fetch official Codex version returned HTTP {}",
            response.status().as_u16()
        ));
    }
    let payload = response
        .json::<Value>()
        .await
        .map_err(|err| format!("parse official Codex version response failed: {err}"))?;
    parse_official_codex_stable_version(&payload)
        .ok_or_else(|| "official Codex version response has no stable version".to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        candidate_catalog_preserves_current_models, client_version_check_is_due,
        client_version_next_check_at, discovered_version_is_newer, effective_version_after_probe,
        effective_version_from_state_and_memory, parse_codex_client_version_state,
        parse_official_codex_stable_version, CodexClientVersionState,
    };

    #[test]
    fn parses_only_official_stable_version() {
        assert_eq!(
            parse_official_codex_stable_version(&json!({"version":"0.144.3"})).as_deref(),
            Some("0.144.3")
        );
        assert_eq!(
            parse_official_codex_stable_version(&json!({"version":"0.145.0-alpha.8"})),
            None
        );
        assert_eq!(parse_official_codex_stable_version(&json!({})), None);
    }

    #[test]
    fn checks_official_version_at_most_once_per_day() {
        let state = CodexClientVersionState {
            version: "0.144.3".to_string(),
            checked_at_unix_secs: 100,
            next_check_at_unix_secs: 100 + 86_400,
        };
        assert!(!client_version_check_is_due(Some(&state), 100 + 86_399));
        assert!(client_version_check_is_due(Some(&state), 100 + 86_400));
        assert!(client_version_check_is_due(None, 100));
    }

    #[test]
    fn failed_checks_retry_after_fifteen_minutes() {
        assert_eq!(client_version_next_check_at(100, true), 100 + 86_400);
        assert_eq!(client_version_next_check_at(100, false), 100 + 900);
    }

    #[test]
    fn probes_only_newer_stable_versions() {
        assert!(discovered_version_is_newer("0.144.1", "0.144.3"));
        assert!(!discovered_version_is_newer("0.144.3", "0.144.3"));
        assert!(!discovered_version_is_newer("0.144.3", "0.143.0"));
        assert!(!discovered_version_is_newer("0.144.3", "0.145.0-alpha.8"));
    }

    #[test]
    fn keeps_previous_version_when_probe_fails() {
        assert_eq!(
            effective_version_after_probe("0.144.1", "0.144.3", false),
            "0.144.1"
        );
        assert_eq!(
            effective_version_after_probe("0.144.1", "0.144.3", true),
            "0.144.3"
        );
    }

    #[test]
    fn candidate_catalog_must_preserve_current_models() {
        assert!(candidate_catalog_preserves_current_models(
            &["gpt-5.5".to_string(), "gpt-5.6-sol".to_string()],
            &[
                "gpt-5.5".to_string(),
                "gpt-5.6-sol".to_string(),
                "gpt-5.6-terra".to_string(),
            ],
        ));
        assert!(!candidate_catalog_preserves_current_models(
            &["gpt-5.5".to_string(), "gpt-5.6-sol".to_string()],
            &["gpt-5.5".to_string()],
        ));
        assert!(!candidate_catalog_preserves_current_models(
            &[],
            &["gpt-5.6-sol".to_string()],
        ));
    }

    #[test]
    fn storage_failure_uses_process_last_known_version() {
        assert_eq!(
            effective_version_from_state_and_memory(None, Some("0.144.3")),
            "0.144.3"
        );
        assert_eq!(
            effective_version_from_state_and_memory(None, None),
            "0.144.1"
        );
    }

    #[test]
    fn rejects_invalid_persisted_client_version_state() {
        let error = parse_codex_client_version_state(json!({
            "version": "latest",
            "checked_at_unix_secs": 100,
            "next_check_at_unix_secs": 200
        }))
        .expect_err("invalid stored version should fail");

        assert_eq!(error, "stored Codex client version is invalid");
    }
}

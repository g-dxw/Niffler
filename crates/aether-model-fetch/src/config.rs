use semver::Version;

const MODEL_FETCH_INTERVAL_MINUTES_DEFAULT: u64 = 1440;
const MODEL_FETCH_INTERVAL_MINUTES_MIN: u64 = 60;
const MODEL_FETCH_INTERVAL_MINUTES_MAX: u64 = 10080;
const MODEL_FETCH_STARTUP_DELAY_SECONDS_DEFAULT: u64 = 10;
pub const CODEX_MODEL_FETCH_CLIENT_VERSION_DEFAULT: &str = "0.144.1";

pub fn is_valid_codex_model_fetch_client_version(value: &str) -> bool {
    Version::parse(value.trim()).is_ok()
}

pub fn codex_model_fetch_client_version_override() -> Option<String> {
    normalize_codex_model_fetch_client_version_override(
        std::env::var("CODEX_MODEL_FETCH_CLIENT_VERSION").ok(),
    )
}

fn normalize_codex_model_fetch_client_version_override(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| is_valid_codex_model_fetch_client_version(value))
}

pub fn codex_model_fetch_client_version() -> String {
    codex_model_fetch_client_version_from_override(codex_model_fetch_client_version_override())
}

fn codex_model_fetch_client_version_from_override(value: Option<String>) -> String {
    value.unwrap_or_else(|| CODEX_MODEL_FETCH_CLIENT_VERSION_DEFAULT.to_string())
}

pub fn model_fetch_interval_minutes() -> u64 {
    std::env::var("MODEL_FETCH_INTERVAL_MINUTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| {
            value.clamp(
                MODEL_FETCH_INTERVAL_MINUTES_MIN,
                MODEL_FETCH_INTERVAL_MINUTES_MAX,
            )
        })
        .unwrap_or(MODEL_FETCH_INTERVAL_MINUTES_DEFAULT)
}

pub fn model_fetch_startup_enabled() -> bool {
    std::env::var("MODEL_FETCH_STARTUP_ENABLED")
        .ok()
        .map(|value| value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(true)
}

pub fn model_fetch_startup_delay_seconds() -> u64 {
    std::env::var("MODEL_FETCH_STARTUP_DELAY_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(MODEL_FETCH_STARTUP_DELAY_SECONDS_DEFAULT)
}

#[cfg(test)]
mod tests {
    use super::{
        codex_model_fetch_client_version_from_override, is_valid_codex_model_fetch_client_version,
        model_fetch_interval_minutes, model_fetch_startup_delay_seconds,
        model_fetch_startup_enabled, normalize_codex_model_fetch_client_version_override,
    };

    struct TestEnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl Drop for TestEnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_deref() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn set_test_env_var(key: &'static str, value: &str) -> TestEnvVarGuard {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        TestEnvVarGuard { key, previous }
    }

    #[test]
    fn interval_minutes_clamps_to_supported_bounds() {
        let _interval = set_test_env_var("MODEL_FETCH_INTERVAL_MINUTES", "5");
        assert_eq!(model_fetch_interval_minutes(), 60);

        let _interval = set_test_env_var("MODEL_FETCH_INTERVAL_MINUTES", "20000");
        assert_eq!(model_fetch_interval_minutes(), 10080);
    }

    #[test]
    fn startup_flags_read_from_environment() {
        let _enabled = set_test_env_var("MODEL_FETCH_STARTUP_ENABLED", "false");
        let _delay = set_test_env_var("MODEL_FETCH_STARTUP_DELAY_SECONDS", "3");
        assert!(!model_fetch_startup_enabled());
        assert_eq!(model_fetch_startup_delay_seconds(), 3);
    }

    #[test]
    fn codex_client_version_defaults_to_current_stable_version() {
        assert_eq!(
            codex_model_fetch_client_version_from_override(None),
            "0.144.1"
        );
    }

    #[test]
    fn codex_client_version_override_accepts_valid_versions() {
        assert_eq!(
            normalize_codex_model_fetch_client_version_override(Some("0.144.3".to_string()))
                .as_deref(),
            Some("0.144.3")
        );
        assert_eq!(
            codex_model_fetch_client_version_from_override(Some("0.144.3".to_string())),
            "0.144.3"
        );
    }

    #[test]
    fn codex_client_version_accepts_manual_prerelease_but_rejects_invalid_values() {
        assert!(is_valid_codex_model_fetch_client_version("0.144.3"));
        assert!(is_valid_codex_model_fetch_client_version("0.145.0-alpha.8"));
        assert!(!is_valid_codex_model_fetch_client_version("latest"));
        assert!(!is_valid_codex_model_fetch_client_version("0.144.3&x=y"));
        assert_eq!(
            normalize_codex_model_fetch_client_version_override(Some("latest".to_string())),
            None
        );
    }
}

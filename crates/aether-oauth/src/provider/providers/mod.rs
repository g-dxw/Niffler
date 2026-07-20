mod antigravity;
mod codex;
mod generic;
mod grok_oauth;
mod kiro;

pub use antigravity::AntigravityProviderOAuthAdapter;
pub use codex::CodexProviderOAuthAdapter;
pub use generic::{
    GenericProviderOAuthAdapter, GenericProviderOAuthTemplate, GENERIC_PROVIDER_OAUTH_TEMPLATES,
};
pub use grok_oauth::{
    apply_grok_oauth_auth_config_defaults, enrich_grok_oauth_auth_config,
    GrokOAuthProviderOAuthAdapter, GROK_OAUTH_PROVIDER_TYPE,
};
pub use kiro::{
    generate_kiro_machine_id, normalize_kiro_machine_id, KiroAuthConfig, KiroProviderOAuthAdapter,
    DEFAULT_KIRO_VERSION, DEFAULT_NODE_VERSION, DEFAULT_REGION, DEFAULT_SYSTEM_VERSION,
    KIRO_PROVIDER_TYPE,
};

use crate::resolver::ResolverConfig;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub history_enabled: bool,
    pub show_sensitive_session: bool,
    pub timeout_seconds: u64,
    pub resolver: ResolverConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            history_enabled: false,
            show_sensitive_session: false,
            timeout_seconds: 10,
            resolver: ResolverConfig::default(),
        }
    }
}

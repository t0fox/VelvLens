use std::{fs, path::Path};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::resolver::ResolverConfig;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub history_enabled: bool,
    pub timeout_seconds: u64,
    pub resolver: ResolverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct PersistedSettings {
    history_enabled: bool,
    timeout_seconds: u64,
    max_depth: u8,
    max_total_bytes: usize,
    max_bytes_per_source: usize,
    max_discovered_urls: usize,
    max_candidates: usize,
    max_redirects: usize,
    max_configs: usize,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        let resolver = ResolverConfig::default();
        Self {
            history_enabled: false,
            timeout_seconds: 10,
            max_depth: resolver.max_depth,
            max_total_bytes: resolver.max_total_bytes,
            max_bytes_per_source: resolver.max_bytes_per_source,
            max_discovered_urls: resolver.max_discovered_urls,
            max_candidates: resolver.max_candidates,
            max_redirects: resolver.max_redirects,
            max_configs: resolver.max_configs,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            history_enabled: false,
            timeout_seconds: 10,
            resolver: ResolverConfig::default(),
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        ProjectDirs::from("com", "SubLens", "SubLens")
            .map(|dirs| Self::load_from_path(dirs.config_dir().join("settings.json")))
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(dirs) = ProjectDirs::from("com", "SubLens", "SubLens") else {
            return Ok(());
        };
        self.save_to_path(dirs.config_dir().join("settings.json"))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Self {
        let mut settings = Self::default();
        if let Ok(bytes) = fs::read(path) {
            if let Ok(persisted) = serde_json::from_slice::<PersistedSettings>(&bytes) {
                settings.history_enabled = persisted.history_enabled;
                settings.timeout_seconds = persisted.timeout_seconds;
                settings.resolver.max_depth = persisted.max_depth;
                settings.resolver.max_total_bytes = persisted.max_total_bytes;
                settings.resolver.max_bytes_per_source = persisted.max_bytes_per_source;
                settings.resolver.max_discovered_urls = persisted.max_discovered_urls;
                settings.resolver.max_candidates = persisted.max_candidates;
                settings.resolver.max_redirects = persisted.max_redirects;
                settings.resolver.max_configs = persisted.max_configs;
            }
        }
        settings.normalize();
        settings
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut normalized = self.clone();
        normalized.normalize();
        let persisted = PersistedSettings {
            history_enabled: normalized.history_enabled,
            timeout_seconds: normalized.timeout_seconds,
            max_depth: normalized.resolver.max_depth,
            max_total_bytes: normalized.resolver.max_total_bytes,
            max_bytes_per_source: normalized.resolver.max_bytes_per_source,
            max_discovered_urls: normalized.resolver.max_discovered_urls,
            max_candidates: normalized.resolver.max_candidates,
            max_redirects: normalized.resolver.max_redirects,
            max_configs: normalized.resolver.max_configs,
        };
        let bytes = serde_json::to_vec_pretty(&persisted).map_err(|error| error.to_string())?;
        fs::write(path, bytes).map_err(|error| error.to_string())
    }

    fn normalize(&mut self) {
        self.timeout_seconds = self.timeout_seconds.clamp(2, 60);
        self.resolver.max_depth = self.resolver.max_depth.clamp(1, 16);
        self.resolver.max_total_bytes = self
            .resolver
            .max_total_bytes
            .clamp(1_048_576, 64 * 1_024 * 1_024);
        self.resolver.max_bytes_per_source = self
            .resolver
            .max_bytes_per_source
            .clamp(256 * 1_024, 16 * 1_024 * 1_024);
        self.resolver.max_discovered_urls = self.resolver.max_discovered_urls.clamp(8, 512);
        self.resolver.max_candidates = self.resolver.max_candidates.clamp(64, 16_384);
        self.resolver.max_redirects = self.resolver.max_redirects.clamp(1, 32);
        self.resolver.max_configs = self.resolver.max_configs.clamp(64, 16_384);
    }
}

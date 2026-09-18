use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::platform;

const MAX_ENTRIES: usize = 12;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct HistoryFile {
    urls: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HistoryStore {
    enabled: bool,
    path: Option<PathBuf>,
    urls: Vec<String>,
}

impl HistoryStore {
    pub fn load(enabled: bool) -> Self {
        let path = ProjectDirs::from("com", "SubLens", "SubLens")
            .map(|dirs| dirs.config_dir().join("history.bin"));
        if !enabled {
            return Self {
                enabled: false,
                path,
                urls: Vec::new(),
            };
        }
        let urls = path
            .as_ref()
            .and_then(|path| fs::read(path).ok())
            .and_then(|bytes| platform::unprotect(&bytes).ok())
            .and_then(|bytes| serde_json::from_slice::<HistoryFile>(&bytes).ok())
            .map(|history| history.urls)
            .unwrap_or_default();
        Self {
            enabled,
            path,
            urls,
        }
    }

    pub fn urls(&self) -> &[String] {
        &self.urls
    }

    pub fn append(&mut self, url: &str) {
        if !self.enabled || url.trim().is_empty() {
            return;
        }
        self.urls.retain(|seen| seen != url);
        self.urls.insert(0, url.to_owned());
        self.urls.truncate(MAX_ENTRIES);
        let _ = self.persist();
    }

    pub fn clear(&mut self) {
        self.urls.clear();
        if let Some(path) = &self.path {
            let _ = fs::remove_file(path);
        }
    }

    fn persist(&self) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let payload = serde_json::to_vec(&HistoryFile {
            urls: self.urls.clone(),
        })
        .map_err(|error| error.to_string())?;
        let protected = platform::protect(&payload)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, protected).map_err(|error| error.to_string())
    }
}

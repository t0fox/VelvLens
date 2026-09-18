use std::time::Duration;

use crate::security::{redact_secret, redact_uri};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageKind {
    Http,
    Redirect,
    Detection,
    Decode,
    Extract,
    Parse,
    Deduplication,
    Diagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    Running,
    Success,
    Failed,
    Skipped,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStage {
    pub kind: StageKind,
    pub status: StageStatus,
    pub source: Option<String>,
    pub preview: String,
    pub found: usize,
    pub duration_ms: u128,
    pub error: Option<String>,
}

impl PipelineStage {
    pub fn success(
        kind: StageKind,
        preview: impl Into<String>,
        found: usize,
        duration: Duration,
    ) -> Self {
        Self {
            kind,
            status: StageStatus::Success,
            source: None,
            preview: sanitize_preview(&preview.into()),
            found,
            duration_ms: duration.as_millis(),
            error: None,
        }
    }

    pub fn failure(kind: StageKind, source: Option<&str>, error: impl Into<String>) -> Self {
        Self {
            kind,
            status: StageStatus::Failed,
            source: source.map(redact_uri),
            preview: String::new(),
            found: 0,
            duration_ms: 0,
            error: Some(sanitize_preview(&error.into())),
        }
    }
}

fn sanitize_preview(value: &str) -> String {
    let sanitized = redact_uri(value);
    ["uuid", "password", "token", "secret", "pbk", "sid"]
        .iter()
        .fold(sanitized, |current, key| {
            if current.to_ascii_lowercase().contains(key) {
                redact_secret(&current)
            } else {
                current
            }
        })
}

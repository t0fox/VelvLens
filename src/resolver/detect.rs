use serde_json::Value;

use super::decode::decode_candidates;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    ProxyList,
    Json,
    JsonConfiguration,
    Html,
    Text,
    Base64Candidate,
    Unknown,
}

pub fn detect_content(bytes: &[u8], content_type: Option<&str>) -> ContentKind {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    let content_type = content_type.unwrap_or_default().to_ascii_lowercase();

    if content_type.contains("html") || looks_like_html(trimmed) {
        return ContentKind::Html;
    }
    if has_proxy_uri(trimmed) {
        return ContentKind::ProxyList;
    }
    if content_type.contains("json") || serde_json::from_str::<Value>(trimmed).is_ok() {
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            if looks_like_ready_configuration(&value) {
                return ContentKind::JsonConfiguration;
            }
        }
        return ContentKind::Json;
    }
    if is_plausible_base64(trimmed) {
        return ContentKind::Base64Candidate;
    }
    if !trimmed.is_empty() && text.is_ascii() {
        return ContentKind::Text;
    }
    ContentKind::Unknown
}

fn looks_like_ready_configuration(value: &Value) -> bool {
    looks_like_ready_configuration_at(value, 0)
}

fn looks_like_ready_configuration_at(value: &Value, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    match value {
        Value::Object(object) => {
            let keys = object
                .keys()
                .map(|key| key.to_ascii_lowercase())
                .collect::<Vec<_>>();
            keys.iter().any(|key| {
                matches!(
                    key.as_str(),
                    "protocol" | "outbounds" | "inbounds" | "streamsettings" | "proxy"
                )
            }) || object
                .values()
                .any(|value| looks_like_ready_configuration_at(value, depth + 1))
        }
        Value::Array(values) => values
            .iter()
            .any(|value| looks_like_ready_configuration_at(value, depth + 1)),
        Value::String(_) | Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

pub fn has_proxy_uri(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        [
            "vless://",
            "vmess://",
            "trojan://",
            "ss://",
            "socks://",
            "socks5://",
            "hysteria://",
            "hysteria2://",
            "hy2://",
            "tuic://",
        ]
        .iter()
        .any(|scheme| line.to_ascii_lowercase().starts_with(scheme))
    })
}

fn looks_like_html(text: &str) -> bool {
    let lower = text.get(..256).unwrap_or(text).to_ascii_lowercase();
    lower.contains("<html")
        || lower.contains("<script")
        || lower.contains("<!doctype html")
        || lower.contains("<body")
}

fn is_plausible_base64(text: &str) -> bool {
    // Subscription encoders may wrap Base64 at line boundaries and some add
    // indentation. A single prose line with spaces must not be normalized into
    // a false Base64 candidate, so only allow internal whitespace for multiline
    // payloads and still require a useful decoded result below.
    let has_internal_whitespace = text
        .lines()
        .any(|line| line.chars().any(|character| character.is_whitespace()));
    if has_internal_whitespace && text.lines().count() < 2 {
        return false;
    }
    let normalized = text
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>();
    if normalized.len() < 16 || normalized.contains("://") {
        return false;
    }
    if !normalized.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=' | b'-' | b'_')
    }) {
        return false;
    }
    !decode_candidates(&normalized).is_empty()
}

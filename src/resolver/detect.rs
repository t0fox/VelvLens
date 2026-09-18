use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    ProxyList,
    Json,
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
    if content_type.contains("json") || serde_json::from_str::<Value>(trimmed).is_ok() {
        return ContentKind::Json;
    }
    if has_proxy_uri(trimmed) {
        return ContentKind::ProxyList;
    }
    if is_plausible_base64(trimmed) {
        return ContentKind::Base64Candidate;
    }
    if !trimmed.is_empty() && text.is_ascii() {
        return ContentKind::Text;
    }
    ContentKind::Unknown
}

pub fn has_proxy_uri(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        [
            "vless://",
            "vmess://",
            "trojan://",
            "ss://",
            "hysteria2://",
            "hy2://",
            "tuic://",
        ]
        .iter()
        .any(|scheme| line.starts_with(scheme))
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
    if text.len() < 16 || text.contains(char::is_whitespace) || text.contains("://") {
        return false;
    }
    text.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=' | b'-' | b'_')
    })
}

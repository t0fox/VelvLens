use std::sync::OnceLock;

use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;

use super::detect::ContentKind;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtractionResult {
    pub proxy_uris: Vec<String>,
    pub nested_urls: Vec<String>,
    pub payloads: Vec<String>,
}

const MAX_JSON_DEPTH: usize = 32;
const MAX_JSON_SOURCES: usize = 4096;
const DEFAULT_MAX_ITEMS: usize = 4096;

pub fn extract_items(text: &str, kind: ContentKind) -> ExtractionResult {
    extract_items_with_limit(text, kind, DEFAULT_MAX_ITEMS)
}

pub(crate) fn extract_items_with_limit(
    text: &str,
    kind: ContentKind,
    max_items: usize,
) -> ExtractionResult {
    let mut sources = vec![text.to_owned()];
    if matches!(kind, ContentKind::Json | ContentKind::JsonConfiguration) {
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            collect_json_strings(&value, &mut sources, max_items);
        }
    }
    if kind == ContentKind::Html {
        let document = Html::parse_document(text);
        if let Ok(selector) = Selector::parse("*") {
            let source_limit = max_items.saturating_add(1).min(MAX_JSON_SOURCES);
            for element in document.select(&selector) {
                if sources.len() >= source_limit {
                    break;
                }
                let tag = element.value().name();
                if matches!(tag, "body" | "script") {
                    let value = element.text().collect::<Vec<_>>().join(" ");
                    if !value.trim().is_empty() {
                        sources.push(value);
                    }
                }
                for (name, value) in element.value().attrs() {
                    if sources.len() >= source_limit {
                        break;
                    }
                    if name.eq_ignore_ascii_case("href")
                        || name.eq_ignore_ascii_case("src")
                        || name.to_ascii_lowercase().starts_with("data-")
                    {
                        sources.push(value.to_owned());
                    }
                }
            }
        }
    }

    let mut result = ExtractionResult::default();
    for source in sources {
        extract_regex_matches(&source, &mut result, max_items);
        if !source.trim().is_empty()
            && !source.contains("://")
            && source.trim().len() >= 16
            && !result.payloads.iter().any(|payload| payload == &source)
            && result_item_count(&result) < max_items
        {
            result.payloads.push(source);
        }
    }
    result
}

fn collect_json_strings(value: &Value, sources: &mut Vec<String>, max_items: usize) {
    collect_json_strings_at(value, sources, 0, max_items);
}

fn collect_json_strings_at(
    value: &Value,
    sources: &mut Vec<String>,
    depth: usize,
    max_items: usize,
) {
    let source_limit = MAX_JSON_SOURCES.min(max_items.saturating_add(1));
    if depth > MAX_JSON_DEPTH || sources.len() >= source_limit {
        return;
    }
    match value {
        Value::String(value) => sources.push(value.clone()),
        Value::Array(values) => {
            for value in values {
                if sources.len() >= source_limit {
                    break;
                }
                collect_json_strings_at(value, sources, depth + 1, max_items);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                if sources.len() >= source_limit {
                    break;
                }
                collect_json_strings_at(value, sources, depth + 1, max_items);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn extract_regex_matches(source: &str, result: &mut ExtractionResult, max_items: usize) {
    static CANDIDATE_RE: OnceLock<Regex> = OnceLock::new();
    let regex = CANDIDATE_RE.get_or_init(|| {
        Regex::new(r#"(?i)(?:[a-z][a-z0-9+.-]*)://[^\s<>'\"`]+"#).expect("candidate regex is valid")
    });

    for matched in regex.find_iter(source) {
        if result_item_count(result) >= max_items {
            break;
        }
        let candidate = matched
            .as_str()
            .trim_end_matches([',', '.', ';', ')', ']', '}'])
            .to_owned();
        let scheme = candidate
            .split_once("://")
            .map(|(scheme, _)| scheme.to_ascii_lowercase());
        if matches!(scheme.as_deref(), Some("http" | "https")) {
            push_unique(&mut result.nested_urls, candidate);
        } else {
            push_unique(&mut result.proxy_uris, candidate);
        }
    }
}

fn result_item_count(result: &ExtractionResult) -> usize {
    result.proxy_uris.len() + result.nested_urls.len() + result.payloads.len()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|seen| seen == &value) {
        values.push(value);
    }
}

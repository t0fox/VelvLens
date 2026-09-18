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

pub fn extract_items(text: &str, kind: ContentKind) -> ExtractionResult {
    let mut sources = vec![text.to_owned()];
    if kind == ContentKind::Json {
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            collect_json_strings(&value, &mut sources);
        }
    }
    if kind == ContentKind::Html {
        let document = Html::parse_document(text);
        if let Ok(selector) =
            Selector::parse("body, script, [data-subscription], [data-url], [href]")
        {
            for element in document.select(&selector) {
                sources.push(element.text().collect::<Vec<_>>().join(" "));
                for value in element.value().attrs().map(|(_, value)| value) {
                    sources.push(value.to_owned());
                }
            }
        }
    }

    let mut result = ExtractionResult::default();
    for source in sources {
        extract_regex_matches(&source, &mut result);
        if !source.trim().is_empty()
            && !source.contains("://")
            && source.trim().len() >= 16
            && !result.payloads.iter().any(|payload| payload == &source)
        {
            result.payloads.push(source);
        }
    }
    result
}

fn collect_json_strings(value: &Value, sources: &mut Vec<String>) {
    match value {
        Value::String(value) => sources.push(value.clone()),
        Value::Array(values) => values
            .iter()
            .for_each(|value| collect_json_strings(value, sources)),
        Value::Object(values) => values
            .values()
            .for_each(|value| collect_json_strings(value, sources)),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn extract_regex_matches(source: &str, result: &mut ExtractionResult) {
    static CANDIDATE_RE: OnceLock<Regex> = OnceLock::new();
    let regex = CANDIDATE_RE.get_or_init(|| {
        Regex::new(r#"(?i)(?:[a-z][a-z0-9+.-]*)://[^\s<>'\"`]+"#).expect("candidate regex is valid")
    });

    for matched in regex.find_iter(source) {
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

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|seen| seen == &value) {
        values.push(value);
    }
}

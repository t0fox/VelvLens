use serde_json::Value;

use crate::{
    error::{Result, SubLensError},
    model::ProxyConfig,
    security::redact_uri,
};

pub fn json_dump(configs: &[ProxyConfig], include_sensitive: bool) -> Result<String> {
    let mut values = Vec::with_capacity(configs.len());
    for config in configs {
        let mut value = serde_json::to_value(config)
            .map_err(|error| SubLensError::Export(error.to_string()))?;
        if !include_sensitive {
            redact_value(&mut value);
        }
        values.push(value);
    }
    serde_json::to_string_pretty(&values).map_err(|error| SubLensError::Export(error.to_string()))
}

fn redact_value(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, current) in object.iter_mut() {
                if key.eq_ignore_ascii_case("original") {
                    redact_original(current);
                } else if key.eq_ignore_ascii_case("share_uri") {
                    redact_share_uri(current);
                } else if key.eq_ignore_ascii_case("raw_uri") {
                    if let Some(text) = current.as_str() {
                        *current = Value::String(redact_raw_payload(text));
                    }
                } else if is_sensitive_key(key, current) {
                    *current = Value::String("••••••".to_owned());
                } else {
                    redact_value(current);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                redact_value(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn redact_original(value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };
    for (variant, payload) in object.iter_mut() {
        if variant.eq_ignore_ascii_case("ShareUri") {
            if let Some(text) = payload.as_str() {
                *payload = Value::String(redact_uri(text));
            }
        } else if variant.eq_ignore_ascii_case("JsonProfile") {
            redact_json_document(payload);
        }
    }
}

fn redact_share_uri(value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };
    for (key, payload) in object.iter_mut() {
        if key.eq_ignore_ascii_case("uri") {
            if let Some(text) = payload.as_str() {
                *payload = Value::String(redact_uri(text));
            }
        } else {
            redact_share_uri(payload);
        }
    }
}

fn redact_json_document(value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };
    for (key, payload) in object.iter_mut() {
        if key.eq_ignore_ascii_case("document") {
            if let Some(text) = payload.as_str() {
                if let Ok(mut document) = serde_json::from_str::<Value>(text) {
                    redact_json_payload(&mut document);
                    if let Ok(redacted) = serde_json::to_string(&document) {
                        *payload = Value::String(redacted);
                    }
                }
            }
        }
    }
}

fn redact_raw_payload(text: &str) -> String {
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if let Ok(mut value) = serde_json::from_str::<Value>(text) {
            redact_json_payload(&mut value);
            if let Ok(redacted) = serde_json::to_string(&value) {
                return redacted;
            }
        }
    }
    redact_uri(text)
}

fn redact_json_payload(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, current) in object.iter_mut() {
                if is_sensitive_key(key, current) {
                    *current = Value::String("••••••".to_owned());
                } else {
                    redact_json_payload(current);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                redact_json_payload(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn is_sensitive_key(key: &str, value: &Value) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "uuid"
            | "username"
            | "password"
            | "pass"
            | "token"
            | "pbk"
            | "sid"
            | "secret"
            | "auth"
            | "privatekey"
    ) || (normalized == "id" && value.as_str().is_some_and(looks_like_uuid))
}

fn looks_like_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23]
            .into_iter()
            .all(|index| bytes[index] == b'-')
        && bytes
            .iter()
            .enumerate()
            .filter(|(index, _)| ![8, 13, 18, 23].contains(index))
            .all(|(_, character)| character.is_ascii_hexdigit())
}

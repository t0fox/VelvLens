use std::collections::BTreeMap;

use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use serde_json::Value;

use crate::{
    error::{Result, SubLensError},
    model::{Protocol, Security, Transport},
};

use super::common::{finish, transport_from_query};

pub fn parse(uri: &str) -> Result<crate::model::ProxyConfig> {
    let payload = uri.strip_prefix("vmess://").unwrap_or_default();
    let json = decode_payload(payload).ok_or_else(|| SubLensError::Parse {
        origin: crate::security::redact_uri(uri),
        message: "VMess payload is not valid Base64 JSON".to_owned(),
    })?;
    let value: Value = serde_json::from_str(&json).map_err(|error| SubLensError::Parse {
        origin: crate::security::redact_uri(uri),
        message: format!("VMess JSON is invalid: {error}"),
    })?;

    let host = string_field(&value, "add").ok_or_else(|| SubLensError::Parse {
        origin: crate::security::redact_uri(uri),
        message: "VMess address is missing".to_owned(),
    })?;
    let port = number_field(&value, "port").ok_or_else(|| SubLensError::Parse {
        origin: crate::security::redact_uri(uri),
        message: "VMess port is missing or invalid".to_owned(),
    })?;
    let mut unknown = BTreeMap::new();
    let known = [
        "ps",
        "add",
        "port",
        "id",
        "scy",
        "tls",
        "sni",
        "net",
        "path",
        "host",
        "serviceName",
        "type",
    ];
    if let Value::Object(fields) = &value {
        for (key, value) in fields {
            if !known
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(key))
            {
                unknown.insert(key.clone(), vec![value.to_string()]);
            }
        }
    }
    let mut query = BTreeMap::new();
    if let Some(net) = string_field(&value, "net") {
        query.insert("net".to_owned(), vec![net]);
    }
    let security = match string_field(&value, "tls")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "tls" | "1" | "true" => Security::Tls,
        "" | "none" | "0" | "false" => Security::None,
        _ => Security::Unknown,
    };
    let transport = transport_from_query(&query, Transport::Tcp);
    Ok(finish(
        Protocol::Vmess,
        string_field(&value, "ps"),
        host,
        port,
        string_field(&value, "id"),
        None,
        None,
        security,
        transport,
        string_field(&value, "sni"),
        None,
        None,
        None,
        None,
        string_field(&value, "scy"),
        string_field(&value, "path"),
        string_field(&value, "host"),
        string_field(&value, "serviceName"),
        string_field(&value, "type"),
        unknown,
        uri,
    ))
}

fn decode_payload(payload: &str) -> Option<String> {
    for engine in [&STANDARD, &STANDARD_NO_PAD, &URL_SAFE, &URL_SAFE_NO_PAD] {
        if let Ok(bytes) = engine.decode(payload) {
            if let Ok(text) = String::from_utf8(bytes) {
                if text.trim_start().starts_with('{') {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|value| match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    })
}

fn number_field(value: &Value, key: &str) -> Option<u16> {
    string_field(value, key)?.parse().ok()
}

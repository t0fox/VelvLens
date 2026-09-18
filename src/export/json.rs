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
    let Some(object) = value.as_object_mut() else {
        return;
    };
    for key in ["uuid", "username", "password", "raw_uri"] {
        if let Some(current) = object.get_mut(key) {
            if let Some(text) = current.as_str() {
                *current = Value::String(if key == "raw_uri" {
                    redact_uri(text)
                } else {
                    "••••••".to_owned()
                });
            }
        }
    }
    if let Some(unknown) = object.get_mut("unknown_params") {
        if let Some(params) = unknown.as_object_mut() {
            for key in [
                "uuid", "password", "pass", "token", "pbk", "sid", "secret", "auth",
            ] {
                if let Some(values) = params.get_mut(key) {
                    *values = Value::Array(vec![Value::String("••••••".to_owned())]);
                }
            }
        }
    }
}

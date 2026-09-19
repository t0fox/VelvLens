use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use serde_json::json;

use crate::model::{ConversionLimitation, ProxyConfig, ShareUriResult};

use super::{finish, missing, unknown_limitations_except};

pub fn serialize(config: &ProxyConfig) -> ShareUriResult {
    let Some(uuid) = config.uuid.as_deref().filter(|value| !value.is_empty()) else {
        return missing("uuid");
    };
    let mut value = json!({
        "v": "2",
        "ps": config.name.clone().unwrap_or_default(),
        "add": config.host,
        "port": config.port,
        "id": uuid,
        "scy": config.encryption.clone().unwrap_or_else(|| "auto".to_owned()),
        "tls": if config.security == crate::model::Security::Tls { "tls" } else { "none" },
        "sni": config.sni,
        "net": config.transport.as_str().to_ascii_lowercase(),
        "path": config.path,
        "host": config.host_header,
        "serviceName": config.service_name,
        "type": config.mode,
    });
    let mut limitations = unknown_limitations_except(config, &[]);
    if config.port_range.is_some() {
        limitations.push(ConversionLimitation::AmbiguousParameter {
            field: "port_range".to_owned(),
        });
    }
    if let Some(object) = value.as_object_mut() {
        object.retain(|_, value| !value.is_null());
    }
    let payload = serde_json::to_vec(&value).expect("VMess payload is serializable");
    finish(
        format!("vmess://{}", STANDARD_NO_PAD.encode(payload)),
        limitations,
    )
}

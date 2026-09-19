use std::collections::BTreeMap;

use crate::model::{ConversionLimitation, ProxyConfig, ShareUriResult};

use super::{append_unknown_query, build_uri, finish, missing, unknown_limitations_except};

pub fn serialize(config: &ProxyConfig) -> ShareUriResult {
    let Some(uuid) = config.uuid.as_deref().filter(|value| !value.is_empty()) else {
        return missing("uuid");
    };
    let mut query = BTreeMap::new();
    insert(&mut query, "encryption", config.encryption.as_deref());
    insert(&mut query, "flow", config.flow.as_deref());
    insert(&mut query, "security", security(config));
    insert(&mut query, "type", Some(config.transport.as_str()));
    insert(&mut query, "sni", config.sni.as_deref());
    insert(&mut query, "fp", config.fingerprint.as_deref());
    insert(&mut query, "pbk", config.reality_public_key.as_deref());
    insert(&mut query, "sid", config.reality_short_id.as_deref());
    insert(&mut query, "path", config.path.as_deref());
    insert(&mut query, "host", config.host_header.as_deref());
    insert(&mut query, "serviceName", config.service_name.as_deref());
    insert(&mut query, "mode", config.mode.as_deref());
    append_unknown_query(config, &mut query, &["headerType", "alpn", "allowInsecure"]);

    let mut limitations =
        unknown_limitations_except(config, &["headerType", "alpn", "allowInsecure"]);
    if config.port_range.is_some() {
        limitations.push(ConversionLimitation::AmbiguousParameter {
            field: "port_range".to_owned(),
        });
    }
    if config.security == crate::model::Security::Reality && config.reality_public_key.is_none() {
        limitations.push(ConversionLimitation::MissingRequired {
            field: "reality_public_key".to_owned(),
        });
    }
    finish(
        build_uri(
            "vless",
            Some(uuid),
            &config.host,
            config.port,
            &query,
            config.name.as_deref(),
        ),
        limitations,
    )
}

fn insert(query: &mut BTreeMap<String, String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty() && *value != "Unknown") {
        query.insert(key.to_owned(), value.to_owned());
    }
}

fn security(config: &ProxyConfig) -> Option<&'static str> {
    match config.security {
        crate::model::Security::Reality => Some("reality"),
        crate::model::Security::Tls => Some("tls"),
        crate::model::Security::None => Some("none"),
        crate::model::Security::Unknown => None,
    }
}

use std::collections::BTreeMap;

use crate::model::{ConversionLimitation, ProxyConfig, ShareUriResult};

use super::{append_unknown_query, build_uri, finish, missing, unknown_limitations_except};

pub fn serialize(config: &ProxyConfig) -> ShareUriResult {
    let Some(password) = config.password.as_deref().filter(|value| !value.is_empty()) else {
        return missing("password");
    };
    let mut query = BTreeMap::new();
    if config.security == crate::model::Security::Tls {
        query.insert("security".to_owned(), "tls".to_owned());
    }
    if config.security == crate::model::Security::Reality {
        query.insert("security".to_owned(), "reality".to_owned());
    }
    insert(&mut query, "sni", config.sni.as_deref());
    insert(&mut query, "type", Some(config.transport.as_str()));
    insert(&mut query, "path", config.path.as_deref());
    insert(&mut query, "host", config.host_header.as_deref());
    insert(&mut query, "serviceName", config.service_name.as_deref());
    insert(&mut query, "mode", config.mode.as_deref());
    append_unknown_query(config, &mut query, &["fp", "alpn"]);
    let mut limitations = unknown_limitations_except(config, &["fp", "alpn"]);
    if config.security == crate::model::Security::Reality {
        limitations.push(ConversionLimitation::UnsupportedParameter {
            field: "reality settings for Trojan".to_owned(),
        });
    }
    finish(
        build_uri(
            "trojan",
            Some(password),
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

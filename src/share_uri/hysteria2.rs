use std::collections::BTreeMap;

use crate::model::{ConversionLimitation, ProxyConfig, ShareUriResult};

use super::{append_unknown_query, build_uri_with_port_spec, finish, unknown_limitations_except};

pub fn serialize(config: &ProxyConfig) -> ShareUriResult {
    let mut query = BTreeMap::new();
    if let Some(sni) = &config.sni {
        query.insert("sni".to_owned(), sni.clone());
    }
    append_unknown_query(
        config,
        &mut query,
        &[
            "alpn",
            "insecure",
            "obfs",
            "obfs-password",
            "pinSHA256",
            "protocol",
            "upmbps",
            "downmbps",
            "mport",
            "mportHopInt",
        ],
    );
    let mut limitations = unknown_limitations_except(
        config,
        &[
            "alpn",
            "insecure",
            "obfs",
            "obfs-password",
            "pinSHA256",
            "protocol",
            "upmbps",
            "downmbps",
            "mport",
            "mportHopInt",
        ],
    );
    if config.fingerprint.is_some() {
        limitations.push(ConversionLimitation::UnsupportedParameter {
            field: "fingerprint".to_owned(),
        });
    }
    let port_spec = config
        .port_range
        .clone()
        .unwrap_or_else(|| config.port.to_string());
    let auth = match (
        config.username.as_deref().filter(|value| !value.is_empty()),
        config.password.as_deref().filter(|value| !value.is_empty()),
    ) {
        (Some(username), Some(password)) => Some(format!("{username}:{password}")),
        (_, password) => password.map(ToOwned::to_owned),
    };
    finish(
        build_uri_with_port_spec(
            "hysteria2",
            auth.as_deref(),
            &config.host,
            &port_spec,
            &query,
            config.name.as_deref(),
        ),
        limitations,
    )
}

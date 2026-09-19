use std::collections::BTreeMap;

use crate::model::{ProxyConfig, ShareUriResult};

use super::{append_unknown_query, build_uri, finish, missing, unknown_limitations_except};

pub fn serialize(config: &ProxyConfig) -> ShareUriResult {
    let Some(uuid) = config.uuid.as_deref().filter(|value| !value.is_empty()) else {
        return missing("uuid");
    };
    let Some(password) = config.password.as_deref().filter(|value| !value.is_empty()) else {
        return missing("password");
    };
    let mut query = BTreeMap::new();
    if let Some(sni) = &config.sni {
        query.insert("sni".to_owned(), sni.clone());
    }
    append_unknown_query(
        config,
        &mut query,
        &["congestion_control", "udp_relay_mode", "zero_rtt_handshake"],
    );
    finish(
        build_uri(
            "tuic",
            Some(&format!("{uuid}:{password}")),
            &config.host,
            config.port,
            &query,
            config.name.as_deref(),
        ),
        unknown_limitations_except(
            config,
            &["congestion_control", "udp_relay_mode", "zero_rtt_handshake"],
        ),
    )
}

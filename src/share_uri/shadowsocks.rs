use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

use crate::model::{ProxyConfig, ShareUriResult};

use super::{build_uri, finish, missing, unknown_limitations_except};

pub fn serialize(config: &ProxyConfig) -> ShareUriResult {
    let Some(method) = config
        .username
        .as_deref()
        .or(config.encryption.as_deref())
        .filter(|value| !value.is_empty())
    else {
        return missing("method");
    };
    let Some(password) = config.password.as_deref().filter(|value| !value.is_empty()) else {
        return missing("password");
    };
    let userinfo = STANDARD_NO_PAD.encode(format!("{method}:{password}"));
    finish(
        build_uri(
            "ss",
            Some(&userinfo),
            &config.host,
            config.port,
            &std::collections::BTreeMap::new(),
            config.name.as_deref(),
        ),
        unknown_limitations_except(config, &[]),
    )
}

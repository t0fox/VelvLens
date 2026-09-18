use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;

use crate::{
    error::{Result, SubLensError},
    model::{Protocol, Security, Transport},
};

use super::common::{
    decode_component, finish, first_query, parse_port, parse_url, query_map, remove_known,
};

pub fn parse(uri: &str) -> Result<crate::model::ProxyConfig> {
    let url = parse_url(uri)?;
    let query = query_map(&url);
    let host = url.host_str().unwrap_or_default().to_owned();
    let port = parse_port(&url, uri)?;
    let (method, password) = if let Some(password) = url.password() {
        (
            decode_component(url.username()),
            Some(decode_component(password)),
        )
    } else if !url.username().is_empty() {
        decode_userinfo(url.username()).ok_or_else(|| SubLensError::Parse {
            origin: crate::security::redact_uri(uri),
            message: "Shadowsocks userinfo is not method:password or valid Base64".to_owned(),
        })?
    } else {
        return Err(SubLensError::Parse {
            origin: crate::security::redact_uri(uri),
            message: "Shadowsocks method and password are missing".to_owned(),
        });
    };
    let known = ["plugin", "plugin-opts", "uot", "remarks"];
    Ok(finish(
        Protocol::Shadowsocks,
        url.fragment().map(decode_component),
        host,
        port,
        None,
        Some(method),
        password,
        if first_query(&query, "tls").is_some() {
            Security::Tls
        } else {
            Security::None
        },
        Transport::Tcp,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        remove_known(query, &known),
        uri,
    ))
}

fn decode_userinfo(value: &str) -> Option<(String, Option<String>)> {
    for engine in [&STANDARD, &URL_SAFE, &URL_SAFE_NO_PAD] {
        if let Ok(bytes) = engine.decode(value) {
            if let Ok(decoded) = String::from_utf8(bytes) {
                if let Some((method, password)) = decoded.split_once(':') {
                    return Some((method.to_owned(), Some(password.to_owned())));
                }
            }
        }
    }
    None
}

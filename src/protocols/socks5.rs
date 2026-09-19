use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;

use crate::{
    error::{Result, SubLensError},
    model::{Protocol, Security, Transport},
};

use super::common::{decode_component, finish, parse_port, parse_url, query_map, remove_known};

pub fn parse(uri: &str) -> Result<crate::model::ProxyConfig> {
    let url = parse_url(uri)?;
    if url.port().is_none() {
        if let Some(payload) = uri.split_once("://").map(|(_, value)| value) {
            if let Some(decoded) = decode_full_link(payload) {
                let normalized = if decoded.contains("://") {
                    decoded
                } else {
                    format!("socks://{decoded}")
                };
                if normalized != uri {
                    return parse(&normalized);
                }
            }
        }
    }
    let (username, password) = if let Some(password) = url.password() {
        (
            Some(decode_component(url.username())),
            Some(decode_component(password)),
        )
    } else if !url.username().is_empty() {
        let userinfo = decode_component(url.username());
        decode_userinfo(&userinfo).ok_or_else(|| SubLensError::Parse {
            origin: crate::security::redact_uri(uri),
            message: "Socks5 userinfo is not user:password or valid Base64".to_owned(),
        })?
    } else {
        (None, None)
    };
    let port = parse_port(&url, uri)?;
    let known = ["remarks", "tag"];
    Ok(finish(
        Protocol::Socks5,
        url.fragment().map(decode_component),
        url.host_str().unwrap_or_default().to_owned(),
        port,
        None,
        username,
        password,
        Security::None,
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
        remove_known(query_map(&url), &known),
        uri,
    ))
}

fn decode_userinfo(value: &str) -> Option<(Option<String>, Option<String>)> {
    for engine in [&STANDARD, &URL_SAFE, &URL_SAFE_NO_PAD] {
        if let Ok(bytes) = engine.decode(value) {
            if let Ok(decoded) = String::from_utf8(bytes) {
                if let Some((username, password)) = decoded.split_once(':') {
                    return Some((Some(username.to_owned()), Some(password.to_owned())));
                }
            }
        }
    }
    None
}

fn decode_full_link(value: &str) -> Option<String> {
    for engine in [&STANDARD, &URL_SAFE, &URL_SAFE_NO_PAD] {
        if let Ok(bytes) = engine.decode(value) {
            if let Ok(decoded) = String::from_utf8(bytes) {
                if decoded.contains('@') && decoded.rsplit_once(':').is_some() {
                    return Some(decoded);
                }
            }
        }
    }
    None
}

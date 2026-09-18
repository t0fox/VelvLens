mod common;
pub mod hysteria2;
pub mod shadowsocks;
pub mod trojan;
pub mod tuic;
pub mod vless;
pub mod vmess;

use url::Url;

use crate::{
    error::{Result, SubLensError},
    model::{Protocol, ProxyConfig, Security, Transport},
};

pub fn is_supported_scheme(uri: &str) -> bool {
    uri.split_once("://").is_some_and(|(scheme, _)| {
        matches!(
            scheme.to_ascii_lowercase().as_str(),
            "vless" | "vmess" | "trojan" | "ss" | "hysteria" | "hysteria2" | "hy2" | "tuic"
        )
    })
}

pub fn parse_uri(uri: &str, source: Option<&Url>, depth: u8) -> Result<ProxyConfig> {
    let scheme = uri
        .split_once("://")
        .map(|(scheme, _)| scheme.to_ascii_lowercase())
        .ok_or_else(|| SubLensError::Parse {
            origin: crate::security::redact_uri(uri),
            message: "URI is missing a scheme separator".to_owned(),
        })?;

    let mut config = match scheme.as_str() {
        "vless" => vless::parse(uri)?,
        "vmess" => vmess::parse(uri)?,
        "trojan" => trojan::parse(uri)?,
        "ss" => shadowsocks::parse(uri)?,
        "hysteria" | "hysteria2" | "hy2" => hysteria2::parse(uri)?,
        "tuic" => tuic::parse(uri)?,
        _ => parse_unknown(uri, source, depth, &scheme)?,
    };
    config.raw_uri = uri.to_owned();
    config.metadata.source_url = source.map(|url| url.to_string());
    config.metadata.depth = depth;
    Ok(config)
}

fn parse_unknown(uri: &str, source: Option<&Url>, depth: u8, scheme: &str) -> Result<ProxyConfig> {
    let url = Url::parse(uri).map_err(|error| SubLensError::Parse {
        origin: crate::security::redact_uri(uri),
        message: format!("invalid URI: {error}"),
    })?;
    let host = url.host_str().unwrap_or("unknown").to_owned();
    let port = url.port().unwrap_or_default();
    let name = url.fragment().map(common::decode_component);
    let mut config = common::finish(
        Protocol::Unknown,
        name,
        host,
        port,
        None,
        None,
        None,
        Security::Unknown,
        Transport::Unknown,
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
        common::query_map(&url),
        uri,
    );
    config.id = format!("{scheme}://{}:{}", config.host, config.port);
    config.metadata.source_url = source.map(|source| source.to_string());
    config.metadata.depth = depth;
    Ok(config)
}

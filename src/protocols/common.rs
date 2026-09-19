use std::collections::BTreeMap;

use percent_encoding::percent_decode_str;
use url::Url;

use crate::{
    error::{Result, SubLensError},
    model::{OriginalRepresentation, ProxyConfig, Security, ShareUriResult, Transport},
};

pub fn parse_url(uri: &str) -> Result<Url> {
    Url::parse(uri).map_err(|error| SubLensError::Parse {
        origin: crate::security::redact_uri(uri),
        message: format!("invalid URI: {error}"),
    })
}

pub fn parse_port(url: &Url, uri: &str) -> Result<u16> {
    url.port().ok_or_else(|| SubLensError::Parse {
        origin: crate::security::redact_uri(uri),
        message: "endpoint port is missing".to_owned(),
    })
}

pub fn decode_component(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}

pub fn query_map(url: &Url) -> BTreeMap<String, Vec<String>> {
    let mut values = BTreeMap::new();
    for (key, value) in url.query_pairs() {
        values
            .entry(key.into_owned())
            .or_insert_with(Vec::new)
            .push(value.into_owned());
    }
    values
}

pub fn first_query(map: &BTreeMap<String, Vec<String>>, key: &str) -> Option<String> {
    map.iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .and_then(|(_, values)| values.first().cloned())
}

pub fn remove_known(
    mut map: BTreeMap<String, Vec<String>>,
    known: &[&str],
) -> BTreeMap<String, Vec<String>> {
    map.retain(|key, _| {
        !known
            .iter()
            .any(|candidate| key.eq_ignore_ascii_case(candidate))
    });
    map
}

pub fn security_from_query(map: &BTreeMap<String, Vec<String>>, default: Security) -> Security {
    match first_query(map, "security")
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "reality" => Security::Reality,
        "tls" | "xtls" => Security::Tls,
        "none" | "" => {
            if first_query(map, "tls")
                .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            {
                Security::Tls
            } else {
                default
            }
        }
        _ => Security::Unknown,
    }
}

pub fn transport_from_query(map: &BTreeMap<String, Vec<String>>, default: Transport) -> Transport {
    let value = first_query(map, "type")
        .or_else(|| first_query(map, "net"))
        .unwrap_or_default();
    match value.to_ascii_lowercase().as_str() {
        "tcp" | "" => default,
        "xhttp" | "splithttp" => Transport::XHttp,
        "ws" | "websocket" => Transport::WebSocket,
        "grpc" => Transport::Grpc,
        "h2" | "http" | "http2" => Transport::Http2,
        "quic" => Transport::Quic,
        _ => Transport::Unknown,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn finish(
    protocol: crate::model::Protocol,
    name: Option<String>,
    host: String,
    port: u16,
    uuid: Option<String>,
    username: Option<String>,
    password: Option<String>,
    security: Security,
    transport: Transport,
    sni: Option<String>,
    fingerprint: Option<String>,
    reality_public_key: Option<String>,
    reality_short_id: Option<String>,
    flow: Option<String>,
    encryption: Option<String>,
    path: Option<String>,
    host_header: Option<String>,
    service_name: Option<String>,
    mode: Option<String>,
    unknown_params: BTreeMap<String, Vec<String>>,
    raw_uri: &str,
) -> ProxyConfig {
    let id = format!(
        "{}://{}:{}",
        protocol.as_str().to_ascii_lowercase(),
        host,
        port
    );
    ProxyConfig {
        id,
        protocol,
        name,
        host,
        port,
        port_range: None,
        uuid,
        username,
        password,
        security,
        transport,
        sni,
        fingerprint,
        reality_public_key,
        reality_short_id,
        flow,
        encryption,
        path,
        host_header,
        service_name,
        mode,
        unknown_params,
        original: OriginalRepresentation::ShareUri(raw_uri.to_owned()),
        share_uri: ShareUriResult::Available {
            uri: raw_uri.to_owned(),
        },
        metadata: Default::default(),
    }
}

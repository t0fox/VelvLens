use std::collections::BTreeMap;

use serde_json::Value;
use url::Url;

use crate::model::{ConfigMetadata, Protocol, ProxyConfig, Security, Transport};

/// Parse ready-made Xray JSON profiles while keeping the original profile JSON
/// in `raw_uri`. This is intentionally an inspector representation: it does
/// not synthesize a lossy share URI from fields that were not present.
pub fn parse_configurations(text: &str, source: &Url, depth: u8) -> Vec<ProxyConfig> {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    let profiles = match value {
        Value::Array(values) => values,
        value => vec![value],
    };
    profiles
        .iter()
        .filter_map(|profile| parse_profile(profile, source, depth))
        .collect()
}

fn parse_profile(profile: &Value, source: &Url, depth: u8) -> Option<ProxyConfig> {
    let outbound = select_outbound(profile)?;
    let protocol: Protocol = protocol_from(string_value(outbound.get("protocol"))?.as_str()).into();
    let (host, port, port_range) = endpoint(outbound)?;
    let settings = outbound.get("settings").unwrap_or(&Value::Null);
    let stream = outbound.get("streamSettings").unwrap_or(&Value::Null);
    let security = security_from(stream, protocol);
    let transport = transport_from(stream, protocol);
    let name = first_string(profile, &["remarks", "name", "tag"]);
    let uuid = first_nested_string(settings, &["vnext", "0", "users", "0", "id"])
        .or_else(|| first_nested_string(settings, &["servers", "0", "uuid"]));
    let username = first_nested_string(settings, &["servers", "0", "users", "0", "user"])
        .or_else(|| first_nested_string(settings, &["servers", "0", "users", "0", "username"]));
    let password = first_nested_string(settings, &["servers", "0", "password"])
        .or_else(|| first_nested_string(settings, &["servers", "0", "users", "0", "pass"]))
        .or_else(|| first_nested_string(settings, &["servers", "0", "users", "0", "password"]))
        .or_else(|| first_nested_string(stream, &["hysteriaSettings", "auth"]))
        .or_else(|| first_nested_string(settings, &["auth"]));
    let encryption = first_nested_string(settings, &["vnext", "0", "users", "0", "encryption"])
        .or_else(|| first_nested_string(settings, &["servers", "0", "method"]));
    let flow = first_nested_string(settings, &["vnext", "0", "users", "0", "flow"]);
    let sni = first_nested_string(stream, &["realitySettings", "serverName"])
        .or_else(|| first_nested_string(stream, &["tlsSettings", "serverName"]))
        .or_else(|| first_nested_string(stream, &["hysteriaSettings", "serverName"]));
    let fingerprint = first_nested_string(stream, &["realitySettings", "fingerprint"])
        .or_else(|| first_nested_string(stream, &["tlsSettings", "fingerprint"]));
    let reality_public_key = first_nested_string(stream, &["realitySettings", "publicKey"]);
    let reality_short_id = first_nested_string(stream, &["realitySettings", "shortId"]);
    let path = first_nested_string(stream, &["wsSettings", "path"])
        .or_else(|| first_nested_string(stream, &["xhttpSettings", "path"]))
        .or_else(|| first_nested_string(stream, &["splithttpSettings", "path"]))
        .or_else(|| first_nested_string(stream, &["httpSettings", "path"]));
    let host_header = first_nested_string(stream, &["wsSettings", "headers", "Host"])
        .or_else(|| first_nested_string(stream, &["wsSettings", "headers", "host"]))
        .or_else(|| first_nested_string(stream, &["httpSettings", "host", "0"]));
    let service_name = first_nested_string(stream, &["grpcSettings", "serviceName"]);
    let mode = first_nested_string(stream, &["grpcSettings", "mode"])
        .or_else(|| first_nested_string(stream, &["xhttpSettings", "mode"]))
        .or_else(|| first_nested_string(stream, &["splithttpSettings", "mode"]));
    let raw_uri = serde_json::to_string(profile).ok()?;

    Some(ProxyConfig {
        id: format!(
            "{}://{}:{}",
            protocol.as_str().to_ascii_lowercase(),
            host,
            port
        ),
        protocol,
        name,
        host,
        port,
        port_range,
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
        unknown_params: BTreeMap::new(),
        raw_uri,
        metadata: ConfigMetadata {
            source_url: Some(source.to_string()),
            depth,
            ..ConfigMetadata::default()
        },
    })
}

fn select_outbound(profile: &Value) -> Option<&Value> {
    let candidates = profile
        .get("outbounds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    candidates
        .filter(|candidate| protocol_from_value(candidate).is_some())
        .find(|candidate| endpoint(candidate).is_some())
        .or_else(|| {
            if protocol_from_value(profile).is_some() && endpoint(profile).is_some() {
                Some(profile)
            } else {
                None
            }
        })
}

fn protocol_from_value(value: &Value) -> Option<Protocol> {
    protocol_from(string_value(value.get("protocol"))?.as_str()).into_option()
}

fn protocol_from(value: &str) -> ProtocolResult {
    match value.to_ascii_lowercase().as_str() {
        "vless" => ProtocolResult::Known(Protocol::Vless),
        "vmess" => ProtocolResult::Known(Protocol::Vmess),
        "trojan" => ProtocolResult::Known(Protocol::Trojan),
        "shadowsocks" | "ss" => ProtocolResult::Known(Protocol::Shadowsocks),
        "socks" | "socks5" => ProtocolResult::Known(Protocol::Socks5),
        "hysteria" | "hysteria2" | "hy2" => ProtocolResult::Known(Protocol::Hysteria2),
        "tuic" => ProtocolResult::Known(Protocol::Tuic),
        _ => ProtocolResult::Unknown,
    }
}

#[derive(Clone, Copy)]
enum ProtocolResult {
    Known(Protocol),
    Unknown,
}

impl ProtocolResult {
    fn into_option(self) -> Option<Protocol> {
        match self {
            Self::Known(protocol) => Some(protocol),
            Self::Unknown => None,
        }
    }
}

impl From<ProtocolResult> for Protocol {
    fn from(value: ProtocolResult) -> Self {
        match value {
            ProtocolResult::Known(protocol) => protocol,
            ProtocolResult::Unknown => Protocol::Unknown,
        }
    }
}

fn endpoint(outbound: &Value) -> Option<(String, u16, Option<String>)> {
    let settings = outbound.get("settings").unwrap_or(&Value::Null);
    let endpoint = first_object(settings, &["vnext", "0"])
        .or_else(|| first_object(settings, &["servers", "0"]))
        .unwrap_or(settings);
    let host = first_string(endpoint, &["address", "add", "host"])?;
    let port_value = first_string(endpoint, &["port"])?;
    let first_port = port_value
        .split([',', '-'])
        .next()
        .and_then(|value| value.parse::<u16>().ok())?;
    let port_range = (port_value.contains(',') || port_value.contains('-'))
        .then_some(port_value)
        .filter(|value| !value.is_empty());
    Some((host, first_port, port_range))
}

fn security_from(stream: &Value, protocol: Protocol) -> Security {
    if stream
        .get("realitySettings")
        .is_some_and(|value| !value.is_null())
    {
        return Security::Reality;
    }
    match first_string(stream, &["security"])
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "reality" => Security::Reality,
        "tls" => Security::Tls,
        "none" | "" if protocol == Protocol::Hysteria2 => Security::Tls,
        "none" | "" => Security::None,
        _ => Security::Unknown,
    }
}

fn transport_from(stream: &Value, protocol: Protocol) -> Transport {
    let network = first_string(stream, &["network"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    match network.as_str() {
        "ws" | "websocket" => Transport::WebSocket,
        "grpc" => Transport::Grpc,
        "xhttp" | "splithttp" => Transport::XHttp,
        "h2" | "http" | "http2" => Transport::Http2,
        "quic" => Transport::Quic,
        "hysteria" | "hysteria2" | "hy2" => Transport::Quic,
        "tcp" | "" if protocol == Protocol::Hysteria2 => Transport::Quic,
        "tcp" | "" => Transport::Tcp,
        _ => Transport::Unknown,
    }
}

fn first_object<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = if let Ok(index) = segment.parse::<usize>() {
            current.as_array()?.get(index)?
        } else {
            current.get(*segment)?
        };
    }
    current.is_object().then_some(current)
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| string_value(value.get(*key)))
}

fn first_nested_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = if let Ok(index) = segment.parse::<usize>() {
            current.as_array()?.get(index)?
        } else {
            current.get(*segment)?
        };
    }
    string_value(Some(current))
}

fn string_value(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_configurations;

    #[test]
    fn parses_hysteria_and_vless_profiles_without_synthesizing_share_uris() {
        let source = url::Url::parse("https://example.test/subscription").unwrap();
        let profiles = parse_configurations(
            include_str!("../../tests/fixtures/ready_xray_profiles.json"),
            &source,
            1,
        );
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].protocol, crate::model::Protocol::Hysteria2);
        assert_eq!(profiles[0].transport, crate::model::Transport::Quic);
        assert_eq!(profiles[1].protocol, crate::model::Protocol::Vless);
        assert_eq!(profiles[0].name.as_deref(), Some("HY2-Torrent"));
        assert!(profiles[0].raw_uri.starts_with('{'));
        assert!(profiles[0].raw_uri.contains("streamSettings"));
    }
}

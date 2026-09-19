use std::{collections::BTreeMap, sync::Arc};

use serde_json::Value;
use url::Url;

use crate::{
    model::{
        ConfigMetadata, ConversionLimitation, OriginalRepresentation, Protocol, ProxyConfig,
        Security, ShareUriResult, Transport,
    },
    share_uri,
};

/// Parse Xray-compatible JSON while keeping one shared source document for all
/// endpoints extracted from it. Every real proxy endpoint gets its own config.
pub fn parse_configurations(text: &str, source: &Url, depth: u8) -> Vec<ProxyConfig> {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    let document: Arc<str> = Arc::from(text.to_owned());
    match value {
        Value::Array(profiles) => profiles
            .iter()
            .enumerate()
            .flat_map(|(profile_index, profile)| {
                parse_profile(profile, document.clone(), profile_index, source, depth)
            })
            .collect(),
        profile => parse_profile(&profile, document, 0, source, depth),
    }
}

fn parse_profile(
    profile: &Value,
    document: Arc<str>,
    profile_index: usize,
    source: &Url,
    depth: u8,
) -> Vec<ProxyConfig> {
    if let Some(outbounds) = profile.get("outbounds").and_then(Value::as_array) {
        return outbounds
            .iter()
            .enumerate()
            .flat_map(|(outbound_index, outbound)| {
                parse_outbound(
                    profile,
                    outbound,
                    document.clone(),
                    profile_index,
                    outbound_index,
                    source,
                    depth,
                )
            })
            .collect();
    }

    if protocol_from_value(profile).is_some() {
        return parse_outbound(profile, profile, document, profile_index, 0, source, depth);
    }
    Vec::new()
}

fn parse_outbound(
    profile: &Value,
    outbound: &Value,
    document: Arc<str>,
    profile_index: usize,
    outbound_index: usize,
    source: &Url,
    depth: u8,
) -> Vec<ProxyConfig> {
    let Some(protocol) = protocol_from_value(outbound) else {
        return Vec::new();
    };
    let candidates = endpoint_candidates(outbound, protocol);
    let total = candidates.len();
    candidates
        .into_iter()
        .enumerate()
        .filter_map(|(endpoint_index, candidate)| {
            parse_endpoint(
                profile,
                outbound,
                candidate,
                document.clone(),
                profile_index,
                outbound_index,
                endpoint_index,
                total,
                protocol,
                source,
                depth,
            )
        })
        .collect()
}

struct EndpointCandidate<'a> {
    endpoint: &'a Value,
    user: Option<&'a Value>,
}

fn endpoint_candidates<'a>(outbound: &'a Value, protocol: Protocol) -> Vec<EndpointCandidate<'a>> {
    let settings = outbound.get("settings").unwrap_or(&Value::Null);
    let mut candidates = Vec::new();

    if let Some(vnext) = settings.get("vnext").and_then(Value::as_array) {
        for server in vnext {
            let users = server
                .get("users")
                .and_then(Value::as_array)
                .filter(|users| !users.is_empty());
            if let Some(users) = users {
                for user in users {
                    candidates.push(EndpointCandidate {
                        endpoint: server,
                        user: Some(user),
                    });
                }
            } else {
                candidates.push(EndpointCandidate {
                    endpoint: server,
                    user: None,
                });
            }
        }
    }

    if let Some(servers) = settings.get("servers").and_then(Value::as_array) {
        for server in servers {
            let users = server
                .get("users")
                .and_then(Value::as_array)
                .filter(|users| !users.is_empty());
            if let Some(users) = users {
                for user in users {
                    candidates.push(EndpointCandidate {
                        endpoint: server,
                        user: Some(user),
                    });
                }
            } else {
                candidates.push(EndpointCandidate {
                    endpoint: server,
                    user: None,
                });
            }
        }
    }

    if candidates.is_empty() && endpoint(outbound, settings, protocol).is_some() {
        candidates.push(EndpointCandidate {
            endpoint: settings,
            user: None,
        });
    }
    candidates
}

#[allow(clippy::too_many_arguments)]
fn parse_endpoint(
    profile: &Value,
    outbound: &Value,
    candidate: EndpointCandidate<'_>,
    document: Arc<str>,
    profile_index: usize,
    outbound_index: usize,
    endpoint_index: usize,
    total: usize,
    protocol: Protocol,
    source: &Url,
    depth: u8,
) -> Option<ProxyConfig> {
    let settings = outbound.get("settings").unwrap_or(&Value::Null);
    let (host, port, port_range) = endpoint(outbound, candidate.endpoint, protocol)?;
    let stream = outbound.get("streamSettings").unwrap_or(&Value::Null);
    let user = candidate.user.unwrap_or(&Value::Null);
    let name = display_name(profile, candidate.endpoint, endpoint_index, total);
    let labels = collect_labels(profile, outbound, candidate.endpoint, candidate.user);
    let uuid = first_string(user, &["id", "uuid"])
        .or_else(|| first_string(candidate.endpoint, &["id", "uuid"]));
    let username = first_string(user, &["user", "username"])
        .or_else(|| first_string(candidate.endpoint, &["user", "username"]));
    let password = first_string(user, &["pass", "password", "auth"])
        .or_else(|| first_string(candidate.endpoint, &["password", "pass", "auth"]))
        .or_else(|| first_nested_string(stream, &["hysteriaSettings", "auth"]))
        .or_else(|| first_string(settings, &["password", "auth"]));
    let encryption = first_string(user, &["encryption", "scy", "method"])
        .or_else(|| first_string(candidate.endpoint, &["encryption", "scy", "method"]));
    let flow =
        first_string(user, &["flow"]).or_else(|| first_string(candidate.endpoint, &["flow"]));
    let security = security_from(stream, protocol);
    let transport = transport_from(stream, protocol);
    let sni = first_nested_string(stream, &["realitySettings", "serverName"])
        .or_else(|| first_nested_string(stream, &["tlsSettings", "serverName"]))
        .or_else(|| first_nested_string(stream, &["hysteriaSettings", "serverName"]))
        .or_else(|| first_string(candidate.endpoint, &["sni"]));
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

    let unknown_params = collect_extra_params(
        outbound,
        settings,
        candidate.endpoint,
        candidate.user,
        stream,
        protocol,
    );
    let mut config = ProxyConfig {
        id: format!(
            "{}://{}:{}#json-p{}-o{}-e{}",
            protocol.as_str().to_ascii_lowercase(),
            host,
            port,
            profile_index,
            outbound_index,
            endpoint_index
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
        unknown_params,
        original: OriginalRepresentation::json_profile(
            document,
            profile_index,
            outbound_index,
            endpoint_index,
        ),
        share_uri: ShareUriResult::Unavailable {
            reasons: vec![ConversionLimitation::UnsupportedParameter {
                field: "share_uri_pending".to_owned(),
            }],
        },
        metadata: ConfigMetadata {
            source_url: Some(source.to_string()),
            depth,
            labels,
            ..ConfigMetadata::default()
        },
    };
    config.share_uri = share_uri::from_json_config(&config);
    Some(config)
}

fn protocol_from_value(value: &Value) -> Option<Protocol> {
    let protocol = string_value(value.get("protocol"))?;
    match protocol.to_ascii_lowercase().as_str() {
        "vless" => Some(Protocol::Vless),
        "vmess" => Some(Protocol::Vmess),
        "trojan" => Some(Protocol::Trojan),
        "shadowsocks" | "ss" => Some(Protocol::Shadowsocks),
        "socks" | "socks5" => Some(Protocol::Socks5),
        "hysteria" | "hysteria2" | "hy2" => Some(Protocol::Hysteria2),
        "tuic" => Some(Protocol::Tuic),
        _ => None,
    }
}

fn endpoint<'a>(
    outbound: &'a Value,
    candidate: &'a Value,
    protocol: Protocol,
) -> Option<(String, u16, Option<String>)> {
    let endpoint = if candidate.is_object() {
        candidate
    } else {
        outbound
    };
    let host = first_string(endpoint, &["address", "add", "host", "server"])?;
    let port_value = first_value(endpoint, &["port"])?;
    let port_spec = value_as_list(port_value)?;
    let first_port = port_spec
        .split([',', '-'])
        .next()
        .and_then(|value| value.trim().parse::<u16>().ok())?;
    let port_range = (port_spec.contains(',') || port_spec.contains('-'))
        .then_some(port_spec)
        .filter(|value| !value.is_empty());
    if protocol == Protocol::Socks5 && first_port == 0 {
        return None;
    }
    Some((host, first_port, port_range))
}

fn display_name(
    profile: &Value,
    endpoint: &Value,
    endpoint_index: usize,
    total: usize,
) -> Option<String> {
    let name = first_string(profile, &["remarks", "name", "tag"])
        .or_else(|| first_string(endpoint, &["remarks", "name", "tag"]));
    name.map(|name| {
        if total > 1 {
            format!("{name} #{}", endpoint_index + 1)
        } else {
            name
        }
    })
}

fn collect_labels(
    profile: &Value,
    outbound: &Value,
    endpoint: &Value,
    user: Option<&Value>,
) -> Vec<String> {
    let mut labels = Vec::new();
    for value in [Some(profile), Some(outbound), Some(endpoint), user] {
        for key in ["remarks", "name", "tag"] {
            if let Some(label) = value.and_then(|value| first_string(value, &[key])) {
                if !label.is_empty() && !labels.contains(&label) {
                    labels.push(label);
                }
            }
        }
    }
    labels
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
        "quic" | "hysteria" | "hysteria2" | "hy2" => Transport::Quic,
        "tcp" | "" if protocol == Protocol::Hysteria2 => Transport::Quic,
        "tcp" | "" => Transport::Tcp,
        _ => Transport::Unknown,
    }
}

fn collect_extra_params(
    outbound: &Value,
    settings: &Value,
    endpoint: &Value,
    user: Option<&Value>,
    stream: &Value,
    protocol: Protocol,
) -> BTreeMap<String, Vec<String>> {
    let mut values = BTreeMap::new();
    let allowed_outbound = [
        "protocol",
        "settings",
        "streamSettings",
        "tag",
        "remarks",
        "name",
    ];
    if let Some(object) = outbound.as_object() {
        for (key, value) in object {
            if !allowed_outbound
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(key))
            {
                values.insert(format!("outbound.{key}"), vec![value.to_string()]);
            }
        }
    }
    let allowed_settings = [
        "address",
        "add",
        "host",
        "server",
        "port",
        "vnext",
        "servers",
        "users",
        "id",
        "uuid",
        "user",
        "username",
        "pass",
        "password",
        "auth",
        "method",
        "encryption",
        "scy",
        "flow",
        "sni",
        "version",
    ];
    if !std::ptr::eq(endpoint, settings) {
        collect_object_extras(&mut values, "settings", settings, &allowed_settings);
    }
    collect_object_extras(
        &mut values,
        "endpoint",
        endpoint,
        endpoint_allowed(protocol),
    );
    if protocol == Protocol::Hysteria2 {
        if let Some(version) = first_string(endpoint, &["version"]) {
            if version != "2" {
                values.insert("endpoint.version".to_owned(), vec![version]);
            }
        }
    }
    if let Some(user) = user {
        collect_object_extras(&mut values, "user", user, user_allowed(protocol));
    }
    let allowed_stream = [
        "network",
        "security",
        "tlsSettings",
        "realitySettings",
        "wsSettings",
        "xhttpSettings",
        "splithttpSettings",
        "httpSettings",
        "grpcSettings",
        "hysteriaSettings",
    ];
    if let Some(object) = stream.as_object() {
        for (key, value) in object {
            if !allowed_stream
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(key))
            {
                values.insert(format!("streamSettings.{key}"), vec![value.to_string()]);
            }
        }
    }
    collect_stream_nested_extras(&mut values, stream, protocol);
    if protocol == Protocol::Hysteria2 {
        if let Some(alpn) = first_nested_list(stream, &["tlsSettings", "alpn"]) {
            values.insert("alpn".to_owned(), vec![alpn]);
        }
        if let Some(version) = first_nested_string(stream, &["hysteriaSettings", "version"]) {
            if version != "2" {
                values.insert("version".to_owned(), vec![version]);
            }
        }
    }
    values
}

fn collect_object_extras(
    values: &mut BTreeMap<String, Vec<String>>,
    prefix: &str,
    value: &Value,
    allowed: &[&str],
) {
    if let Some(object) = value.as_object() {
        for (key, value) in object {
            if !allowed
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(key))
            {
                values.insert(format!("{prefix}.{key}"), vec![value.to_string()]);
            }
        }
    }
}

fn endpoint_allowed(protocol: Protocol) -> &'static [&'static str] {
    match protocol {
        Protocol::Vless => &[
            "address",
            "add",
            "host",
            "server",
            "port",
            "users",
            "id",
            "uuid",
            "user",
            "username",
            "encryption",
            "scy",
            "flow",
            "sni",
            "remarks",
            "name",
            "tag",
        ],
        Protocol::Vmess => &[
            "address",
            "add",
            "host",
            "server",
            "port",
            "users",
            "id",
            "uuid",
            "user",
            "username",
            "encryption",
            "scy",
            "flow",
            "sni",
            "remarks",
            "name",
            "tag",
        ],
        Protocol::Trojan => &[
            "address", "add", "host", "server", "port", "users", "password", "pass", "auth", "sni",
            "remarks", "name", "tag",
        ],
        Protocol::Shadowsocks => &[
            "address", "add", "host", "server", "port", "method", "password", "pass", "remarks",
            "name", "tag",
        ],
        Protocol::Hysteria2 => &[
            "address", "add", "host", "server", "port", "version", "password", "pass", "auth",
            "username", "sni", "remarks", "name", "tag",
        ],
        Protocol::Tuic => &[
            "address", "add", "host", "server", "port", "uuid", "password", "pass", "sni",
            "remarks", "name", "tag",
        ],
        Protocol::Socks5 | Protocol::Unknown => &[
            "address", "add", "host", "server", "port", "users", "user", "username", "password",
            "pass",
        ],
    }
}

fn user_allowed(protocol: Protocol) -> &'static [&'static str] {
    match protocol {
        Protocol::Vless => &["id", "uuid", "flow", "encryption"],
        Protocol::Vmess => &["id", "uuid"],
        Protocol::Trojan => &["password", "pass"],
        Protocol::Shadowsocks => &["method", "password", "pass"],
        Protocol::Hysteria2 => &["password", "pass", "auth", "user", "username"],
        Protocol::Tuic => &["uuid", "password", "pass"],
        Protocol::Socks5 | Protocol::Unknown => &["user", "username", "password", "pass"],
    }
}

fn collect_stream_nested_extras(
    values: &mut BTreeMap<String, Vec<String>>,
    stream: &Value,
    protocol: Protocol,
) {
    let nested_allowed: &[(&str, &[&str])] = &[
        (
            "tlsSettings",
            &[
                "serverName",
                "fingerprint",
                "alpn",
                "allowInsecure",
                "minVersion",
                "maxVersion",
                "cipherSuites",
                "certificates",
                "disableSystemRoot",
                "enableSessionResumption",
                "show",
                "xver",
                "publicKey",
                "shortId",
                "spiderX",
                "echConfigList",
            ],
        ),
        (
            "realitySettings",
            &[
                "serverName",
                "fingerprint",
                "publicKey",
                "shortId",
                "spiderX",
                "show",
                "xver",
            ],
        ),
        (
            "wsSettings",
            &["path", "headers", "maxEarlyData", "earlyDataHeaderName"],
        ),
        (
            "xhttpSettings",
            &["path", "mode", "host", "headers", "scMaxEachPostBytes"],
        ),
        (
            "splithttpSettings",
            &["path", "mode", "host", "headers", "scMaxEachPostBytes"],
        ),
        (
            "httpSettings",
            &["path", "host", "method", "headers", "read_idle_timeout"],
        ),
        ("grpcSettings", &["serviceName", "mode", "multiMode"]),
        ("hysteriaSettings", &["auth", "version", "serverName"]),
    ];
    for (section, allowed) in nested_allowed {
        if let Some(value) = stream.get(*section) {
            collect_object_extras(values, &format!("streamSettings.{section}"), value, allowed);
        }
    }
    if protocol != Protocol::Hysteria2 {
        if let Some(alpn) = first_nested_list(stream, &["tlsSettings", "alpn"]) {
            values.insert("streamSettings.tlsSettings.alpn".to_owned(), vec![alpn]);
        }
    }
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

fn first_nested_list(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = if let Ok(index) = segment.parse::<usize>() {
            current.as_array()?.get(index)?
        } else {
            current.get(*segment)?
        };
    }
    value_as_list(current)
}

fn first_value<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| value.get(*key))
}

fn string_value(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn value_as_list(value: &Value) -> Option<String> {
    match value {
        Value::Array(values) => Some(
            values
                .iter()
                .filter_map(|value| string_value(Some(value)))
                .collect::<Vec<_>>()
                .join(","),
        ),
        _ => string_value(Some(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_configurations;

    #[test]
    fn parses_every_proxy_in_ready_profiles() {
        let source = url::Url::parse("https://example.test/subscription").unwrap();
        let profiles = parse_configurations(
            include_str!("../../tests/fixtures/ready_xray_profiles.json"),
            &source,
            1,
        );
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].protocol, crate::model::Protocol::Hysteria2);
        assert_eq!(profiles[1].protocol, crate::model::Protocol::Vless);
        assert!(profiles[0].original_is_json());
        assert!(profiles[0].share_uri.uri().is_some());
    }
}

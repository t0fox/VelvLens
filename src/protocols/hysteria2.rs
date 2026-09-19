use crate::{
    error::{Result, SubLensError},
    model::{Protocol, Security, Transport},
};

use super::common::{
    decode_component, finish, first_query, parse_port, parse_url, query_map, remove_known,
    security_from_query,
};

pub fn parse(uri: &str) -> Result<crate::model::ProxyConfig> {
    let url = parse_url(uri)?;
    let query = query_map(&url);
    let port_spec = first_query(&query, "port");
    let (port, port_range) = match port_spec.as_deref() {
        Some(spec) => {
            let first = spec.split(',').next().unwrap_or_default();
            let first_port = first.split('-').next().unwrap_or_default();
            let port = first_port.parse::<u16>().map_err(|_| SubLensError::Parse {
                origin: crate::security::redact_uri(uri),
                message: format!("invalid Hysteria2 port specification: {spec}"),
            })?;
            let range = spec.contains(',') || spec.contains('-');
            (port, range.then(|| spec.to_owned()))
        }
        None => (parse_port(&url, uri)?, None),
    };
    let known = [
        "sni",
        "security",
        "tls",
        "alpn",
        "insecure",
        "obfs",
        "obfs-password",
        "auth",
        "protocol",
        "upmbps",
        "downmbps",
        "port",
        "mportHopInt",
    ];
    let mut config = finish(
        Protocol::Hysteria2,
        url.fragment().map(decode_component),
        url.host_str().unwrap_or_default().to_owned(),
        port,
        None,
        None,
        url.password()
            .map(decode_component)
            .or_else(|| (!url.username().is_empty()).then(|| decode_component(url.username())))
            .or_else(|| first_query(&query, "auth").map(|value| decode_component(&value))),
        security_from_query(&query, Security::Tls),
        Transport::Quic,
        first_query(&query, "sni"),
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
    );
    config.port_range = port_range;
    Ok(config)
}

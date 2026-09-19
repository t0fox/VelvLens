use crate::{
    error::{Result, SubLensError},
    model::{Protocol, Security, Transport},
};

use super::common::{
    decode_component, finish, first_query, parse_port, parse_url, query_map, remove_known,
    security_from_query,
};

pub fn parse(uri: &str) -> Result<crate::model::ProxyConfig> {
    let authority_port_range = authority_port_range(uri);
    let url = match parse_url(uri) {
        Ok(url) => url,
        Err(_) if authority_port_range.is_some() => parse_url(&replace_authority_port(
            uri,
            authority_port_range.as_deref().unwrap(),
        ))?,
        Err(error) => return Err(error),
    };
    let query = query_map(&url);
    let port_spec = first_query(&query, "port").or(authority_port_range);
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
    let (username, password) = if let Some(password) = url.password() {
        (
            (!url.username().is_empty()).then(|| decode_component(url.username())),
            Some(decode_component(password)),
        )
    } else {
        (
            None,
            (!url.username().is_empty())
                .then(|| decode_component(url.username()))
                .or_else(|| first_query(&query, "auth").map(|value| decode_component(&value))),
        )
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
        username,
        password,
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

fn authority_port_range(uri: &str) -> Option<String> {
    let (start, end) = authority_port_span(uri)?;
    let port = &uri[start..end];
    (port.contains(',') || port.contains('-')).then(|| port.to_owned())
}

fn replace_authority_port(uri: &str, port_range: &str) -> String {
    let (start, end) = authority_port_span(uri).expect("authority port range was detected");
    let first_port = port_range
        .split([',', '-'])
        .next()
        .expect("port range has a first port");
    format!("{}{}{}", &uri[..start], first_port, &uri[end..])
}

fn authority_port_span(uri: &str) -> Option<(usize, usize)> {
    let scheme_end = uri.find("://")?.checked_add(3)?;
    let authority_end = uri[scheme_end..]
        .find(['/', '?', '#'])
        .map(|offset| scheme_end + offset)
        .unwrap_or(uri.len());
    let authority = &uri[scheme_end..authority_end];
    let host_start = authority
        .rfind('@')
        .map(|offset| scheme_end + offset + 1)
        .unwrap_or(scheme_end);
    let host_port = &uri[host_start..authority_end];
    if host_port.starts_with('[') {
        let closing_bracket = host_port.find(']')?;
        let colon = closing_bracket + 1;
        if host_port.as_bytes().get(colon) != Some(&b':') {
            return None;
        }
        return Some((host_start + colon + 1, authority_end));
    }
    let colon = host_port.rfind(':')?;
    Some((host_start + colon + 1, authority_end))
}

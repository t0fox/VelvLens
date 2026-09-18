use crate::{
    error::Result,
    model::{Protocol, Security, Transport},
};

use super::common::{
    decode_component, finish, first_query, parse_port, parse_url, query_map, remove_known,
    security_from_query,
};

pub fn parse(uri: &str) -> Result<crate::model::ProxyConfig> {
    let url = parse_url(uri)?;
    let query = query_map(&url);
    let known = [
        "sni",
        "security",
        "tls",
        "congestion_control",
        "udp_relay_mode",
        "zero_rtt_handshake",
    ];
    Ok(finish(
        Protocol::Tuic,
        url.fragment().map(decode_component),
        url.host_str().unwrap_or_default().to_owned(),
        parse_port(&url, uri)?,
        (!url.username().is_empty()).then(|| decode_component(url.username())),
        None,
        url.password().map(decode_component),
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
    ))
}

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
        "alpn",
        "insecure",
        "obfs",
        "obfs-password",
        "auth",
        "protocol",
        "upmbps",
        "downmbps",
    ];
    Ok(finish(
        Protocol::Hysteria2,
        url.fragment().map(decode_component),
        url.host_str().unwrap_or_default().to_owned(),
        parse_port(&url, uri)?,
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
    ))
}

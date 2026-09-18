use crate::{
    error::Result,
    model::{Protocol, Security, Transport},
};

use super::common::{
    decode_component, finish, first_query, parse_port, parse_url, query_map, remove_known,
    security_from_query, transport_from_query,
};

pub fn parse(uri: &str) -> Result<crate::model::ProxyConfig> {
    let url = parse_url(uri)?;
    let query = query_map(&url);
    let known = [
        "security",
        "tls",
        "sni",
        "type",
        "net",
        "path",
        "host",
        "serviceName",
        "mode",
    ];
    Ok(finish(
        Protocol::Trojan,
        url.fragment().map(decode_component),
        url.host_str().unwrap_or_default().to_owned(),
        parse_port(&url, uri)?,
        None,
        None,
        url.password()
            .map(decode_component)
            .or_else(|| (!url.username().is_empty()).then(|| decode_component(url.username()))),
        security_from_query(&query, Security::Tls),
        transport_from_query(&query, Transport::Tcp),
        first_query(&query, "sni"),
        first_query(&query, "fp"),
        None,
        None,
        None,
        None,
        first_query(&query, "path"),
        first_query(&query, "host"),
        first_query(&query, "serviceName"),
        first_query(&query, "mode"),
        remove_known(query, &known),
        uri,
    ))
}

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
        "flow",
        "type",
        "sni",
        "fp",
        "pbk",
        "sid",
        "path",
        "host",
        "serviceName",
        "mode",
        "encryption",
        "headerType",
        "tls",
    ];
    let transport = transport_from_query(&query, Transport::Tcp);
    let security = security_from_query(&query, Security::None);
    Ok(finish(
        Protocol::Vless,
        url.fragment().map(decode_component),
        url.host_str().unwrap_or_default().to_owned(),
        parse_port(&url, uri)?,
        (!url.username().is_empty()).then(|| decode_component(url.username())),
        None,
        None,
        security,
        transport,
        first_query(&query, "sni"),
        first_query(&query, "fp"),
        first_query(&query, "pbk"),
        first_query(&query, "sid"),
        first_query(&query, "flow"),
        first_query(&query, "encryption"),
        first_query(&query, "path"),
        first_query(&query, "host"),
        first_query(&query, "serviceName"),
        first_query(&query, "mode"),
        remove_known(query, &known),
        uri,
    ))
}

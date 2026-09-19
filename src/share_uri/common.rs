use std::collections::BTreeMap;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::model::ProxyConfig;

pub fn host_for_uri(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_owned()
    }
}

pub fn build_uri(
    scheme: &str,
    userinfo: Option<&str>,
    host: &str,
    port: u16,
    query: &BTreeMap<String, String>,
    fragment: Option<&str>,
) -> String {
    build_uri_with_port_spec(scheme, userinfo, host, &port.to_string(), query, fragment)
}

pub fn build_uri_with_port_spec(
    scheme: &str,
    userinfo: Option<&str>,
    host: &str,
    port_spec: &str,
    query: &BTreeMap<String, String>,
    fragment: Option<&str>,
) -> String {
    let mut uri = format!("{scheme}://");
    if let Some(userinfo) = userinfo.filter(|value| !value.is_empty()) {
        uri.push_str(
            &userinfo
                .split_once(':')
                .map(|(left, right)| {
                    format!("{}:{}", encode_component(left), encode_component(right))
                })
                .unwrap_or_else(|| encode_component(userinfo)),
        );
        uri.push('@');
    }
    uri.push_str(&host_for_uri(host));
    uri.push(':');
    uri.push_str(port_spec);
    if !query.is_empty() {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        for (key, value) in query {
            serializer.append_pair(key, value);
        }
        uri.push('?');
        uri.push_str(&serializer.finish());
    }
    if let Some(fragment) = fragment.filter(|value| !value.is_empty()) {
        uri.push('#');
        uri.push_str(&encode_component(fragment));
    }
    uri
}

pub fn append_unknown_query(
    config: &ProxyConfig,
    query: &mut BTreeMap<String, String>,
    allowed: &[&str],
) {
    for (key, values) in &config.unknown_params {
        if allowed
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(key))
        {
            if let Some(value) = values.first() {
                query.insert(key.clone(), value.clone());
            }
        }
    }
}

fn encode_component(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

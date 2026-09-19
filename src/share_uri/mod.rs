mod common;
mod hysteria2;
mod shadowsocks;
mod trojan;
mod tuic;
mod vless;
mod vmess;

use crate::model::{Protocol, ProxyConfig, ShareUriResult};

pub fn from_json_config(config: &ProxyConfig) -> ShareUriResult {
    match config.protocol {
        Protocol::Vless => vless::serialize(config),
        Protocol::Vmess => vmess::serialize(config),
        Protocol::Trojan => trojan::serialize(config),
        Protocol::Shadowsocks => shadowsocks::serialize(config),
        Protocol::Hysteria2 => hysteria2::serialize(config),
        Protocol::Tuic => tuic::serialize(config),
        Protocol::Socks5 | Protocol::Unknown => ShareUriResult::Unavailable {
            reasons: vec![crate::model::ConversionLimitation::UnsupportedParameter {
                field: config.protocol.as_str().to_owned(),
            }],
        },
    }
}

fn finish(uri: String, limitations: Vec<crate::model::ConversionLimitation>) -> ShareUriResult {
    if limitations.is_empty() {
        ShareUriResult::Available { uri }
    } else {
        ShareUriResult::Limited { uri, limitations }
    }
}

fn missing(field: &str) -> ShareUriResult {
    ShareUriResult::Unavailable {
        reasons: vec![crate::model::ConversionLimitation::MissingRequired {
            field: field.to_owned(),
        }],
    }
}

pub(crate) use common::{append_unknown_query, build_uri, build_uri_with_port_spec};

pub(crate) fn unknown_limitations_except(
    config: &ProxyConfig,
    supported: &[&str],
) -> Vec<crate::model::ConversionLimitation> {
    config
        .unknown_params
        .keys()
        .filter(|field| {
            !supported
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(field))
        })
        .map(
            |field| crate::model::ConversionLimitation::UnsupportedParameter {
                field: field.clone(),
            },
        )
        .collect()
}

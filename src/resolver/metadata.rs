use std::collections::BTreeMap;

use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use reqwest::header::HeaderMap;

const PRESERVED_HEADER_DIRECTIVES: [&str; 18] = [
    "profile-title",
    "profile-update-interval",
    "subscription-userinfo",
    "support-url",
    "change-user-agent",
    "profile-web-page-url",
    "announce",
    "subscriptions-sort-type",
    "exclude-local-networks-enable",
    "exclude-apns-enable",
    "dont-use-filter",
    "subscription-pin",
    "manual-block-user-agent",
    "dns-from-json-enable",
    "user-agent-geo-files",
    "proxy-ping-timeout",
    "hide-vpn-icon",
    "subscription-request-timeout",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubscriptionMetadata {
    pub profile_title: Option<String>,
    pub profile_update_interval: Option<u64>,
    pub subscription_userinfo: Option<String>,
    pub support_url: Option<String>,
    pub directives: BTreeMap<String, String>,
}

impl SubscriptionMetadata {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let mut metadata = Self::default();
        for key in PRESERVED_HEADER_DIRECTIVES {
            if let Some(value) = headers.get(key).and_then(|value| value.to_str().ok()) {
                let value = value.trim();
                if !value.is_empty() {
                    metadata.directives.insert(key.to_owned(), value.to_owned());
                }
            }
        }
        for (key, slot) in [
            ("profile-title", &mut metadata.profile_title),
            ("subscription-userinfo", &mut metadata.subscription_userinfo),
            ("support-url", &mut metadata.support_url),
        ] {
            if let Some(raw) = headers.get(key).and_then(|value| value.to_str().ok()) {
                let value = raw.trim();
                if !value.is_empty() {
                    *slot = Some(value.to_owned());
                }
            }
        }
        if let Some(value) = headers
            .get("profile-update-interval")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok())
        {
            metadata.profile_update_interval = Some(value);
        }
        if let Some(title) = metadata.profile_title.take() {
            metadata.profile_title = Some(decode_title(&title));
        }
        metadata
    }

    pub fn merge(&mut self, other: Self) {
        if other.profile_title.is_some() {
            self.profile_title = other.profile_title;
        }
        if other.profile_update_interval.is_some() {
            self.profile_update_interval = other.profile_update_interval;
        }
        if other.subscription_userinfo.is_some() {
            self.subscription_userinfo = other.subscription_userinfo;
        }
        if other.support_url.is_some() {
            self.support_url = other.support_url;
        }
        self.directives.extend(other.directives);
    }
}

pub fn parse_body_directives(text: &str) -> SubscriptionMetadata {
    let mut metadata = SubscriptionMetadata::default();
    for line in text.lines() {
        let Some((key, value)) = line
            .trim()
            .strip_prefix('#')
            .and_then(|line| line.split_once(':'))
        else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        metadata.directives.insert(key.clone(), value.to_owned());
        match key.as_str() {
            "profile-title" => metadata.profile_title = Some(decode_title(value)),
            "profile-update-interval" => {
                metadata.profile_update_interval = value.parse::<u64>().ok();
            }
            "subscription-userinfo" => metadata.subscription_userinfo = Some(value.to_owned()),
            "support-url" => metadata.support_url = Some(value.to_owned()),
            _ => {}
        }
    }
    metadata
}

fn decode_title(value: &str) -> String {
    for engine in [&STANDARD, &URL_SAFE, &URL_SAFE_NO_PAD] {
        if let Ok(bytes) = engine.decode(value) {
            if let Ok(decoded) = String::from_utf8(bytes) {
                if decoded.chars().all(|character| !character.is_control()) {
                    return decoded;
                }
            }
        }
    }
    value.to_owned()
}

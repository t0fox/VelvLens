use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::model::ProxyConfig;

pub fn base64_subscription(configs: &[ProxyConfig]) -> String {
    STANDARD.encode(crate::export::raw::share_uri_lines(configs))
}

use crate::model::ProxyConfig;

pub fn v2rayn_bulk(configs: &[ProxyConfig]) -> String {
    crate::export::raw::raw_lines(configs)
}

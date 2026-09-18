use crate::model::ProxyConfig;

pub fn raw_lines(configs: &[ProxyConfig]) -> String {
    let mut output = configs
        .iter()
        .map(|config| config.raw_uri.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

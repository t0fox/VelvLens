use std::collections::HashSet;

use crate::model::{ConversionLimitation, ProxyConfig, ShareUriResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitedShareExport {
    pub id: String,
    pub uri: String,
    pub limitations: Vec<ConversionLimitation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShareExportSummary {
    pub available: Vec<String>,
    pub limited: Vec<LimitedShareExport>,
    pub unavailable: Vec<(String, Vec<ConversionLimitation>)>,
}

pub fn share_export_summary(configs: &[ProxyConfig]) -> ShareExportSummary {
    let mut summary = ShareExportSummary::default();
    let mut seen_semantic_keys = HashSet::new();
    for config in configs {
        if !seen_semantic_keys.insert(config.semantic_key()) {
            continue;
        }
        match &config.share_uri {
            ShareUriResult::Available { uri } => summary.available.push(uri.clone()),
            ShareUriResult::Limited { uri, limitations } => {
                summary.limited.push(LimitedShareExport {
                    id: config.id.clone(),
                    uri: uri.clone(),
                    limitations: limitations.clone(),
                })
            }
            ShareUriResult::Unavailable { reasons } => {
                summary
                    .unavailable
                    .push((config.id.clone(), reasons.clone()));
            }
        }
    }
    summary
}

pub fn share_uri_lines(configs: &[ProxyConfig]) -> String {
    let mut output = share_export_summary(configs).available.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

pub fn limited_share_uri_lines(configs: &[ProxyConfig]) -> String {
    let mut output = share_export_summary(configs)
        .limited
        .into_iter()
        .map(|entry| entry.uri)
        .collect::<Vec<_>>()
        .join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

pub fn original_json_documents(configs: &[ProxyConfig]) -> String {
    let mut seen = std::collections::HashSet::new();
    let documents = configs
        .iter()
        .filter(|config| config.original_is_json())
        .map(ProxyConfig::original_text)
        .filter(|document| seen.insert((*document).to_owned()))
        .collect::<Vec<_>>();
    if documents.is_empty() {
        String::new()
    } else {
        format!("{}\n", documents.join("\n\n"))
    }
}

pub fn raw_lines(configs: &[ProxyConfig]) -> String {
    let mut output = configs
        .iter()
        .map(|config| config.original_text())
        .collect::<Vec<_>>()
        .join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

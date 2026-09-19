use crate::{
    model::Protocol,
    resolver::{stage::StageKind, AnalysisReport},
};

pub fn format_summary(_source: &str, report: &AnalysisReport) -> String {
    let formats = report
        .stages
        .iter()
        .filter(|stage| stage.kind == StageKind::Detection)
        .map(|stage| stage.preview.as_str())
        .filter(|preview| !preview.is_empty())
        .collect::<Vec<_>>();
    let decode_stages = report
        .stages
        .iter()
        .filter(|stage| stage.kind == StageKind::Decode)
        .count();
    let errors = report
        .stages
        .iter()
        .filter(|stage| stage.error.is_some())
        .count();
    let mut lines = vec![
        "HTTP: PASS".to_owned(),
        format!(
            "format: {}",
            if formats.is_empty() {
                "unknown".to_owned()
            } else {
                formats.join(" -> ")
            }
        ),
        format!("decode stages: {decode_stages}"),
        format!("configs found: {}", report.configs.len()),
    ];
    for protocol in Protocol::ALL {
        let count = report
            .protocol_counts
            .get(protocol.as_str())
            .copied()
            .unwrap_or_default();
        lines.push(format!("{}: {count}", protocol.as_str()));
    }
    lines.push(format!(
        "duplicates: exact {}, semantic {}",
        report.duplicates.exact_duplicates, report.duplicates.semantic_duplicates
    ));
    lines.push(format!("errors: {errors}"));
    lines.join("\n")
}

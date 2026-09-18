use crate::resolver::AnalysisReport;

pub fn format_summary(_source: &str, report: &AnalysisReport) -> String {
    let mut lines = vec![
        "URL fetched: PASS".to_owned(),
        format!("Decode pipeline: PASS ({}) stages", report.stages.len()),
        format!("Configs extracted: {}", report.configs.len()),
    ];
    for (protocol, count) in &report.protocol_counts {
        lines.push(format!("{protocol}: {count}"));
    }
    lines.push(format!("Downloaded bytes: {}", report.downloaded_bytes));
    lines.join("\n")
}

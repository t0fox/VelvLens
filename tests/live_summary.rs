use sublens::{live::format_summary, resolver::AnalysisReport};

#[test]
fn live_summary_never_prints_source_or_credentials() {
    let report = AnalysisReport::default();
    let summary = format_summary("https://example.test/private-token", &report);
    assert!(!summary.contains("private-token"));
    assert!(!summary.contains("https://"));
    assert!(summary.contains("VLESS: 0"));
    assert!(summary.contains("Hysteria2: 0"));
}

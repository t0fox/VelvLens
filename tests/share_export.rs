use sublens::{
    export::{original_json_documents, share_export_summary, share_uri_lines},
    model::Protocol,
    protocols::parse_uri,
    resolver::xray::parse_configurations,
};

#[test]
fn share_export_contains_only_standard_uris_and_counts_losses() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let mut configs = vec![parse_uri("vless://uuid@example.test:443#URI", None, 0).unwrap()];
    configs.extend(parse_configurations(
        include_str!("fixtures/ready_xray_profiles.json"),
        &source,
        0,
    ));

    let summary = share_export_summary(&configs);
    assert_eq!(summary.available.len(), 2);
    assert_eq!(summary.limited.len(), 1);
    assert_eq!(summary.unavailable.len(), 0);

    let lines = share_uri_lines(&configs);
    assert!(lines.lines().all(|line| line.contains("://")));
    assert!(!lines.contains("\"outbounds\""));
    assert!(lines.contains("vless://uuid@example.test:443#URI"));
    assert!(configs
        .iter()
        .any(|config| config.protocol == Protocol::Hysteria2));
}

#[test]
fn original_json_export_is_separate_and_deduplicated_by_source_document() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let configs = parse_configurations(
        include_str!("fixtures/ready_xray_profiles.json"),
        &source,
        0,
    );

    let documents = original_json_documents(&configs);
    assert!(documents.contains("\"outbounds\""));
    assert_eq!(documents.matches("HY2-Torrent").count(), 1);
    assert_eq!(documents.matches("NL-SMART").count(), 1);
}

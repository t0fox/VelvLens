use sublens::{
    dedup::deduplicate,
    model::Protocol,
    resolver::{
        decode::decode_candidates,
        detect::{detect_content, ContentKind},
        extract::extract_items,
    },
};

#[test]
fn fixture_matrix_covers_plain_encoded_and_invalid_payloads() {
    let plain = include_str!("fixtures/plain_vless.txt");
    assert_eq!(
        detect_content(plain.as_bytes(), Some("text/plain")),
        ContentKind::ProxyList
    );
    assert_eq!(
        extract_items(plain, ContentKind::ProxyList)
            .proxy_uris
            .len(),
        1
    );

    let standard = include_str!("fixtures/base64_subscription.txt");
    assert!(decode_candidates(standard)
        .iter()
        .any(|candidate| candidate.text.starts_with("vless://")));

    let url_safe = include_str!("fixtures/base64url_no_padding.txt");
    assert!(decode_candidates(url_safe)
        .iter()
        .any(|candidate| candidate.text.contains("?x=1")));

    let invalid = include_str!("fixtures/bad_base64.txt");
    assert!(decode_candidates(invalid).is_empty());
    assert_eq!(
        detect_content(invalid.as_bytes(), Some("text/plain")),
        ContentKind::Text
    );
}

#[test]
fn line_wrapped_base64_is_classified_as_a_base64_candidate() {
    let encoded = include_str!("fixtures/base64_subscription.txt");
    let wrapped = encoded
        .as_bytes()
        .chunks(12)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\r\n");
    assert_eq!(
        detect_content(wrapped.as_bytes(), Some("text/plain")),
        ContentKind::Base64Candidate
    );
}

#[test]
fn indented_line_wrapped_base64_is_classified_as_a_base64_candidate() {
    let encoded = include_str!("fixtures/base64_subscription.txt");
    let wrapped = encoded
        .as_bytes()
        .chunks(12)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\r\n  ");
    assert_eq!(
        detect_content(wrapped.as_bytes(), Some("text/plain")),
        ContentKind::Base64Candidate
    );
}

#[test]
fn readable_text_with_spaces_is_not_classified_as_base64() {
    let text = "this is a normal readable subscription note";
    assert_ne!(
        detect_content(text.as_bytes(), Some("text/plain")),
        ContentKind::Base64Candidate
    );
}

#[test]
fn readable_ascii_identifier_is_not_classified_as_base64() {
    let text = "subscription-status-ready-for-review";
    assert_ne!(
        detect_content(text.as_bytes(), Some("text/plain")),
        ContentKind::Base64Candidate
    );
}

#[test]
fn fixture_matrix_preserves_unknown_schemes_exactly() {
    let source = include_str!("fixtures/unknown_scheme.txt").trim();
    let extracted = extract_items(source, ContentKind::Text);
    assert_eq!(extracted.proxy_uris, vec![source]);
    let config = sublens::protocols::parse_uri(source, None, 0).unwrap();
    assert_eq!(config.protocol, Protocol::Unknown);
    assert_eq!(config.raw_uri, source);
}

#[test]
fn fixture_matrix_records_exact_and_semantic_duplicate_groups() {
    let mut exact = parse_fixture_lines(include_str!("fixtures/duplicate_uris.txt"));
    let exact_report = deduplicate(&mut exact);
    assert_eq!(exact_report.exact_groups, vec![vec![0, 1]]);
    assert_eq!(exact[0].metadata.exact_duplicate_count, 2);

    let mut semantic = parse_fixture_lines(include_str!("fixtures/semantic_duplicates.txt"));
    let semantic_report = deduplicate(&mut semantic);
    assert_eq!(semantic_report.semantic_groups, vec![vec![0, 1]]);
    assert_eq!(semantic[0].metadata.semantic_duplicate_group, Some(0));
}

#[test]
fn nested_json_fixture_stays_bounded() {
    let deep_payload = "deep-payload-that-must-not-be-walked";
    let mut value = serde_json::json!({"payload": deep_payload});
    for _ in 0..64 {
        value = serde_json::json!({"next": value});
    }
    let text = serde_json::to_string(&value).unwrap();
    let extracted = extract_items(&text, ContentKind::Json);
    assert!(!extracted
        .payloads
        .iter()
        .any(|payload| payload == deep_payload));
}

#[test]
fn json_string_collection_is_bounded() {
    let values = (0..5000)
        .map(|index| format!("payload-{index:04}-with-enough-length"))
        .collect::<Vec<_>>();
    let text = serde_json::to_string(&serde_json::json!({"items": values})).unwrap();
    let extracted = extract_items(&text, ContentKind::Json);

    assert!(extracted.payloads.len() <= 4096);
}

#[test]
fn uri_extraction_is_bounded() {
    let text = (0..5000)
        .map(|index| format!("vless://00000000-0000-4000-8000-{index:012}@host-{index}.test:443"))
        .collect::<Vec<_>>()
        .join("\n");
    let extracted = extract_items(&text, ContentKind::ProxyList);

    assert!(extracted.proxy_uris.len() <= 4096);
}

#[test]
fn supported_fixture_keeps_hysteria_raw_uri() {
    let source = include_str!("fixtures/happ_formats.txt");
    let hysteria_uri = source
        .lines()
        .find(|line| line.starts_with("hysteria2://"))
        .unwrap();
    let config = sublens::protocols::parse_uri(hysteria_uri, None, 0).unwrap();
    assert_eq!(config.protocol, Protocol::Hysteria2);
    assert_eq!(config.raw_uri, hysteria_uri);
}

fn parse_fixture_lines(text: &str) -> Vec<sublens::model::ProxyConfig> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|uri| sublens::protocols::parse_uri(uri.trim(), None, 0).unwrap())
        .collect()
}

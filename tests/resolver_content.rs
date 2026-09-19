use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;

use sublens::resolver::decode::decode_candidates;
use sublens::resolver::detect::{detect_content, ContentKind};
use sublens::resolver::extract::extract_items;
use sublens::resolver::metadata::parse_body_directives;

#[test]
fn decodes_standard_and_unpadded_url_safe_base64() {
    let value = "vless://12345678-1234-1234-1234-123456789abc@example.com:443";
    let standard = STANDARD.encode(value);
    let url_safe = URL_SAFE_NO_PAD.encode(value);

    assert!(decode_candidates(&standard)
        .iter()
        .any(|candidate| candidate.text.contains("vless://")));
    assert!(decode_candidates(&url_safe)
        .iter()
        .any(|candidate| candidate.text.contains("vless://")));
}

#[test]
fn decodes_base64_with_line_wrapping() {
    let value = "vless://12345678-1234-1234-1234-123456789abc@example.com:443";
    let encoded = STANDARD.encode(value);
    let wrapped = encoded
        .as_bytes()
        .chunks(17)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .expect("base64 chunks are valid UTF-8")
        .join("\r\n  ");

    assert!(decode_candidates(&wrapped)
        .iter()
        .any(|candidate| candidate.text.contains("vless://")));
}

#[test]
fn detects_json_html_proxy_lists_and_base64_candidates() {
    assert_eq!(
        detect_content(
            b"{\"items\":[\"vless://example\"]}",
            Some("application/json")
        ),
        ContentKind::Json
    );
    assert_eq!(
        detect_content(b"<html><body>source</body></html>", None),
        ContentKind::Html
    );
    assert_eq!(
        detect_content(include_bytes!("fixtures/raw_mixed.txt"), Some("text/plain")),
        ContentKind::ProxyList
    );
    assert_eq!(
        detect_content(
            STANDARD.encode("vless://example").as_bytes(),
            Some("text/plain")
        ),
        ContentKind::Base64Candidate
    );
}

#[test]
fn classifies_ready_made_xray_json_without_fabricating_a_uri() {
    let json = include_bytes!("fixtures/ready_xray.json");
    assert_eq!(
        detect_content(json, Some("application/json")),
        ContentKind::JsonConfiguration
    );
    let extracted = extract_items(
        std::str::from_utf8(json).unwrap(),
        ContentKind::JsonConfiguration,
    );
    assert!(extracted.proxy_uris.is_empty());
}

#[test]
fn happ_style_fixture_keeps_metadata_and_all_supported_uri_families() {
    let fixture = include_str!("fixtures/happ_formats.txt");
    let metadata = parse_body_directives(fixture);
    assert_eq!(metadata.profile_title.as_deref(), Some("Primary profile"));
    let extracted = extract_items(fixture, ContentKind::ProxyList);
    let protocols = extracted
        .proxy_uris
        .iter()
        .filter_map(|uri| sublens::protocols::parse_uri(uri, None, 0).ok())
        .map(|config| config.protocol)
        .collect::<std::collections::HashSet<_>>();
    assert!(protocols.contains(&sublens::model::Protocol::Socks5));
    assert!(protocols.contains(&sublens::model::Protocol::Hysteria2));
    assert!(protocols.contains(&sublens::model::Protocol::Vmess));
}

#[test]
fn walks_nested_json_and_html_for_proxy_and_subscription_urls() {
    let json = include_str!("fixtures/nested.json");
    let result = extract_items(json, ContentKind::Json);
    assert!(result
        .proxy_uris
        .iter()
        .any(|uri| uri.starts_with("trojan://")));
    assert!(result
        .nested_urls
        .iter()
        .any(|url| url.starts_with("https://")));

    let html = include_str!("fixtures/embedded.html");
    let result = extract_items(html, ContentKind::Html);
    assert!(result
        .proxy_uris
        .iter()
        .any(|uri| uri.starts_with("vless://")));
    assert!(result
        .nested_urls
        .iter()
        .any(|url| url.starts_with("https://")));
}

#[test]
fn extracts_proxy_uris_from_arbitrary_html_data_attributes() {
    let html = r#"<section data-config="hysteria2://synthetic-password@hy.example:443?sni=hy.example"></section>"#;
    let result = extract_items(html, ContentKind::Html);
    assert_eq!(result.proxy_uris.len(), 1);
    assert!(result.proxy_uris[0].starts_with("hysteria2://"));
}

#[test]
fn extracts_legacy_hysteria_alias_and_case_insensitive_schemes() {
    let result = extract_items(
        "HYSTERIA://hy.example:443?auth=synthetic-password\n",
        ContentKind::ProxyList,
    );
    assert_eq!(result.proxy_uris.len(), 1);
    assert!(result.proxy_uris[0].starts_with("HYSTERIA://"));
}

#[test]
fn preserves_unknown_uri_candidates_for_inspection() {
    let result = extract_items("torrent://tracker.example:6969#sample\n", ContentKind::Text);
    assert_eq!(
        result.proxy_uris,
        vec!["torrent://tracker.example:6969#sample"]
    );

    let config = sublens::protocols::parse_uri(&result.proxy_uris[0], None, 0).unwrap();
    assert_eq!(config.protocol, sublens::model::Protocol::Unknown);
    assert_eq!(config.host, "tracker.example");
    assert_eq!(config.port, 6969);
    assert_eq!(config.name.as_deref(), Some("sample"));
}

#[test]
fn failed_stage_redacts_uri_credentials() {
    let stage = sublens::resolver::stage::PipelineStage::failure(
        sublens::resolver::stage::StageKind::Parse,
        Some("vless://12345678-1234-1234-1234-123456789abc@example.com:443?pbk=secret"),
        "invalid pbk=secret",
    );
    let rendered = format!("{stage:?}");
    assert!(!rendered.contains("123456789abc"));
    assert!(!rendered.contains("secret"));
}

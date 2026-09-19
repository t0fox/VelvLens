use std::time::Duration;

use base64::Engine;

use sublens::{
    export::{base64_subscription, json_dump, raw_lines},
    model::ProxyConfig,
    qr::encode,
};

fn fixture_config() -> ProxyConfig {
    sublens::protocols::parse_uri(
        "vless://12345678-1234-1234-1234-123456789abc@example.com:443?security=reality&pbk=private-key&x-custom=value#Demo",
        None,
        0,
    )
    .unwrap()
}

#[test]
fn raw_export_is_line_oriented_and_preserves_unknown_params() {
    let config = fixture_config();
    let text = raw_lines(std::slice::from_ref(&config));
    assert!(text.ends_with('\n'));
    assert!(text.contains("x-custom=value"));
}

#[test]
fn sanitized_json_hides_credentials() {
    let config = fixture_config();
    let json = json_dump(std::slice::from_ref(&config), false).unwrap();
    assert!(json.contains("••••••"));
    assert!(!json.contains("123456789abc"));
}

#[test]
fn sanitized_json_redacts_credentials_inside_ready_json_payloads() {
    let source = url::Url::parse("https://example.test/subscription").unwrap();
    let config = sublens::resolver::xray::parse_configurations(
        include_str!("fixtures/ready_xray_profiles.json"),
        &source,
        0,
    )
    .into_iter()
    .next()
    .unwrap();

    let json = json_dump(std::slice::from_ref(&config), false).unwrap();
    assert!(!json.contains("synthetic-password"));
    assert!(json.contains("hysteriaSettings"));
    assert!(json.contains("••••••"));
}

#[test]
fn sensitive_json_export_keeps_ready_json_payload_exact() {
    let source = url::Url::parse("https://example.test/subscription").unwrap();
    let config = sublens::resolver::xray::parse_configurations(
        include_str!("fixtures/ready_xray_profiles.json"),
        &source,
        0,
    )
    .into_iter()
    .next()
    .unwrap();

    let json = json_dump(std::slice::from_ref(&config), true).unwrap();
    assert!(json.contains("synthetic-password"));
}

#[test]
fn base64_export_decodes_to_raw_lines() {
    let config = fixture_config();
    let encoded = base64_subscription(std::slice::from_ref(&config));
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .unwrap();
    assert_eq!(String::from_utf8(decoded).unwrap(), raw_lines(&[config]));
}

#[test]
fn qr_matrix_is_local_and_nonempty() {
    let matrix = encode("vless://example.com:443");
    assert!(matrix.width() > 0);
    assert_eq!(matrix.modules().len(), matrix.width() * matrix.width());
    assert!(matrix.modules().iter().any(|cell| *cell));
}

#[tokio::test]
async fn diagnostics_reports_connectivity_only_for_local_listener() {
    use tokio::net::TcpListener;
    use tokio_util::sync::CancellationToken;

    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let config = sublens::protocols::parse_uri(
        &format!("vless://12345678-1234-1234-1234-123456789abc@127.0.0.1:{port}"),
        None,
        0,
    )
    .unwrap();
    let result =
        sublens::diagnostics::check(&config, Duration::from_secs(2), &CancellationToken::new())
            .await
            .unwrap();
    assert_eq!(result.dns, sublens::diagnostics::CheckStatus::Passed);
    assert_eq!(result.tcp, sublens::diagnostics::CheckStatus::Passed);
    assert!(result.connectivity_only);
}

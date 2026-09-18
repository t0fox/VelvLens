use std::time::Duration;

use base64::Engine;

use sublens::{
    export::{base64_subscription, json_dump, v2rayn_bulk},
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
fn v2rayn_export_is_line_oriented_and_preserves_unknown_params() {
    let config = fixture_config();
    let text = v2rayn_bulk(std::slice::from_ref(&config));
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
fn base64_export_decodes_to_v2rayn_lines() {
    let config = fixture_config();
    let encoded = base64_subscription(std::slice::from_ref(&config));
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .unwrap();
    assert_eq!(String::from_utf8(decoded).unwrap(), v2rayn_bulk(&[config]));
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

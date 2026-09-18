use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use sublens::model::{Protocol, Security, Transport};
use sublens::protocols::parse_uri;

#[test]
fn parses_vless_reality_xhttp_and_preserves_unknown_query() {
    let config = parse_uri(
        "vless://12345678-1234-1234-1234-123456789abc@example.com:443/name?security=reality&type=xhttp&sni=example.org&fp=chrome&pbk=public&sid=abcd&x-custom=value",
        None,
        0,
    )
    .unwrap();
    assert_eq!(config.protocol, Protocol::Vless);
    assert_eq!(config.security, Security::Reality);
    assert_eq!(config.transport, Transport::XHttp);
    assert_eq!(config.unknown_params["x-custom"], vec!["value"]);
    assert_eq!(config.reality_public_key.as_deref(), Some("public"));
}

#[test]
fn parses_all_required_schemes_without_crashing_on_extra_parameters() {
    let vmess_payload = STANDARD.encode(r#"{"ps":"VMess","add":"vmess.example","port":443,"id":"12345678-1234-1234-1234-123456789abc","net":"ws","tls":"tls","path":"/"}"#);
    let fixtures = [
        "vless://12345678-1234-1234-1234-123456789abc@example.com:443",
        &format!("vmess://{vmess_payload}"),
        "trojan://synthetic-password@example.net:443?security=tls",
        "ss://method:password@example.org:443",
        "hysteria2://synthetic-password@hy.example:443?sni=hy.example",
        "hysteria://hy.example:443?auth=synthetic-password&sni=hy.example",
        "tuic://12345678-1234-1234-1234-123456789abc:synthetic-password@tuic.example:443",
    ];
    for uri in fixtures {
        let config = parse_uri(uri, None, 0).unwrap();
        assert_ne!(config.protocol, Protocol::Unknown);
        assert!(!config.host.is_empty());
    }
    let legacy_hysteria = parse_uri(
        "hysteria://hy.example:443?auth=synthetic-password&sni=hy.example",
        None,
        0,
    )
    .unwrap();
    assert_eq!(legacy_hysteria.protocol, Protocol::Hysteria2);
    assert_eq!(
        legacy_hysteria.password.as_deref(),
        Some("synthetic-password")
    );
}

#[test]
fn decodes_vmess_json_payload() {
    let payload = STANDARD.encode(r#"{"ps":"VMess","add":"example.com","port":443,"id":"12345678-1234-1234-1234-123456789abc","net":"ws","tls":"tls"}"#);
    let config = parse_uri(&format!("vmess://{payload}"), None, 0).unwrap();
    assert_eq!(config.protocol, Protocol::Vmess);
    assert_eq!(config.port, 443);
    assert_eq!(config.security, Security::Tls);
    assert_eq!(config.transport, Transport::WebSocket);
}

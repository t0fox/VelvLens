use std::collections::BTreeMap;

use sublens::model::{ConfigMetadata, Protocol, ProxyConfig, Security, Transport};
use sublens::security::{redact_secret, redact_uri};

fn fixture_config(name: &str, host: &str, port: u16) -> ProxyConfig {
    ProxyConfig {
        id: format!("{host}:{port}"),
        protocol: Protocol::Vless,
        name: Some(name.to_owned()),
        host: host.to_owned(),
        port,
        port_range: None,
        uuid: Some("12345678-1234-1234-1234-123456789abc".to_owned()),
        username: None,
        password: None,
        security: Security::Tls,
        transport: Transport::Tcp,
        sni: Some("example.com".to_owned()),
        fingerprint: Some("chrome".to_owned()),
        reality_public_key: None,
        reality_short_id: None,
        flow: None,
        encryption: None,
        path: None,
        host_header: None,
        service_name: None,
        mode: None,
        unknown_params: BTreeMap::new(),
        raw_uri: format!("vless://{host}:{port}"),
        metadata: ConfigMetadata::default(),
    }
}

#[test]
fn redaction_hides_credentials_and_keeps_shape() {
    assert_eq!(redact_secret("1234567890abcdef"), "12••••••ef");
    assert_eq!(
        redact_uri("vless://12345678-1234-1234-1234-123456789abc@example.com:443?pbk=private-key"),
        "vless://••••••••@example.com:443?pbk=••••••••"
    );
    assert_eq!(
        redact_uri("https://subscription.example/private-token?auth=secret"),
        "https://subscription.example/••••••?auth=••••••••"
    );
}

#[test]
fn semantic_key_excludes_display_name_but_tracks_endpoint() {
    let left = fixture_config("name-a", "example.com", 443);
    let right = fixture_config("name-b", "example.com", 443);
    assert_eq!(left.semantic_key(), right.semantic_key());
}

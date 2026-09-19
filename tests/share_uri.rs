use sublens::{
    model::{Protocol, ShareUriResult},
    protocols::parse_uri,
    resolver::xray::parse_configurations,
};

#[test]
fn vless_json_endpoint_round_trips_through_generated_share_uri() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let config = parse_configurations(
        r#"{
            "remarks":"Reality endpoint",
            "outbounds":[{"protocol":"vless","settings":{"vnext":[{"address":"edge.example","port":443,"users":[{"id":"00000000-0000-4000-8000-000000000001","encryption":"none","flow":"xtls-rprx-vision"}]}]},"streamSettings":{"network":"tcp","security":"reality","realitySettings":{"serverName":"example.test","fingerprint":"chrome","publicKey":"key","shortId":"0001"}}}]
        }"#,
        &source,
        0,
    )
    .pop()
    .unwrap();

    let uri = match &config.share_uri {
        ShareUriResult::Available { uri } => uri,
        result => panic!("expected exact VLESS share URI, got {result:?}"),
    };
    let parsed = parse_uri(uri, None, 0).unwrap();
    assert_eq!(parsed.protocol, Protocol::Vless);
    assert_eq!(parsed.host, config.host);
    assert_eq!(parsed.port, config.port);
    assert_eq!(parsed.uuid, config.uuid);
    assert_eq!(parsed.security, config.security);
    assert_eq!(parsed.sni, config.sni);
    assert_eq!(parsed.reality_public_key, config.reality_public_key);
}

#[test]
fn hysteria_json_with_vendor_fingerprint_is_limited_not_exact() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let config = parse_configurations(
        include_str!("fixtures/ready_xray_profiles.json"),
        &source,
        0,
    )
    .into_iter()
    .find(|config| config.protocol == Protocol::Hysteria2)
    .unwrap();

    let ShareUriResult::Limited { uri, limitations } = &config.share_uri else {
        panic!("Hysteria2 vendor field must not be declared exact");
    };
    assert!(uri.starts_with("hysteria2://"));
    assert!(limitations
        .iter()
        .any(|limitation| limitation.to_string().contains("fingerprint")));
}

#[test]
fn hysteria_json_port_hopping_uses_the_standard_authority_form() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let config = parse_configurations(
        r#"{
            "remarks":"Port hopping",
            "outbounds":[{"protocol":"hysteria","settings":{"address":"edge.example","port":"443,5000-6000","version":2},"streamSettings":{"network":"hysteria","hysteriaSettings":{"auth":"secret"},"tlsSettings":{"serverName":"edge.example"}}}]
        }"#,
        &source,
        0,
    )
    .pop()
    .unwrap();

    let uri = config.share_uri.uri().unwrap();
    assert!(uri.contains("edge.example:443,5000-6000"));
    assert!(!uri.contains("?port="));
    let parsed = parse_uri(uri, None, 0).unwrap();
    assert_eq!(parsed.port, 443);
    assert_eq!(parsed.port_range.as_deref(), Some("443,5000-6000"));
}

#[test]
fn hysteria_json_userpass_auth_is_percent_encoded_and_round_trips() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let config = parse_configurations(
        r#"{
            "outbounds":[{"protocol":"hysteria","settings":{"address":"edge.example","port":443,"username":"user","password":"p@ss:word","version":2}}]
        }"#,
        &source,
        0,
    )
    .pop()
    .unwrap();

    let uri = config.share_uri.uri().unwrap();
    assert!(uri.contains("user:p%40ss%3Aword@"));
    let parsed = parse_uri(uri, None, 0).unwrap();
    assert_eq!(parsed.username.as_deref(), Some("user"));
    assert_eq!(parsed.password.as_deref(), Some("p@ss:word"));
}

#[test]
fn ordinary_hy2_alias_is_preserved_without_normalizing_the_scheme() {
    let source = "hy2://secret@edge.example:443?sni=edge.example#Alias";
    let config = parse_uri(source, None, 0).unwrap();
    assert!(matches!(
        config.share_uri,
        ShareUriResult::Available { uri } if uri == source
    ));
}

#[test]
fn json_share_serializers_round_trip_supported_protocol_endpoints() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let configs = parse_configurations(
        r#"[
            {"protocol":"vless","settings":{"vnext":[{"address":"vless.example","port":443,"users":[{"id":"00000000-0000-4000-8000-000000000001"}]}]}},
            {"protocol":"vmess","settings":{"vnext":[{"address":"vmess.example","port":443,"users":[{"id":"00000000-0000-4000-8000-000000000002"}]}]}},
            {"protocol":"trojan","settings":{"servers":[{"address":"trojan.example","port":443,"password":"trojan-secret"}]},"streamSettings":{"security":"tls"}},
            {"protocol":"shadowsocks","settings":{"servers":[{"address":"ss.example","port":8388,"method":"aes-128-gcm","password":"ss-secret"}]}},
            {"protocol":"tuic","settings":{"address":"tuic.example","port":443,"uuid":"00000000-0000-4000-8000-000000000003","password":"tuic-secret"}}
        ]"#,
        &source,
        0,
    );

    assert_eq!(configs.len(), 5);
    for config in configs {
        let uri = match &config.share_uri {
            ShareUriResult::Available { uri } => uri,
            result => panic!(
                "expected an available {} URI, got {result:?}",
                config.protocol.as_str()
            ),
        };
        let parsed = parse_uri(uri, None, 0).unwrap();
        assert_eq!(parsed.protocol, config.protocol);
        assert_eq!(parsed.host, config.host);
        assert_eq!(parsed.port, config.port);
    }
}

use std::sync::Arc;

use sublens::{
    model::{OriginalRepresentation, Protocol, ShareUriResult},
    resolver::xray::parse_configurations,
};

#[test]
fn json_profile_extracts_each_proxy_outbound_and_ignores_service_outbounds() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let json = include_str!("fixtures/ready_xray_multi.json");

    let configs = parse_configurations(json, &source, 0);

    assert_eq!(configs.len(), 4);
    assert_eq!(configs[0].protocol, Protocol::Vless);
    assert_eq!(configs[3].protocol, Protocol::Hysteria2);
    assert!(configs.iter().all(|config| config.original_is_json()));
    assert!(configs.iter().all(|config| config.original_text() == json));
    assert!(configs
        .iter()
        .all(|config| { matches!(config.original, OriginalRepresentation::JsonProfile { .. }) }));
    if let (
        OriginalRepresentation::JsonProfile { document: left, .. },
        OriginalRepresentation::JsonProfile {
            document: right, ..
        },
    ) = (&configs[0].original, &configs[1].original)
    {
        assert!(Arc::ptr_eq(left, right));
    } else {
        panic!("expected JSON source representations");
    }
}

#[test]
fn json_hysteria_profile_has_a_share_uri_instead_of_json_raw_payload() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let config = parse_configurations(
        include_str!("fixtures/ready_xray_profiles.json"),
        &source,
        0,
    )
    .into_iter()
    .find(|config| config.protocol == Protocol::Hysteria2)
    .unwrap();

    assert!(matches!(
        config.share_uri,
        ShareUriResult::Available { .. } | ShareUriResult::Limited { .. }
    ));
    let uri = config.share_uri.uri().unwrap();
    assert!(uri.starts_with("hysteria2://"));
    assert!(!uri.trim_start().starts_with('{'));
}

#[test]
fn json_resolver_expands_each_server_and_user_in_one_outbound() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let configs = parse_configurations(
        r#"{
            "outbounds":[{"protocol":"vless","settings":{"vnext":[
                {"address":"one.example","port":443,"users":[{"id":"00000000-0000-4000-8000-000000000001"},{"id":"00000000-0000-4000-8000-000000000002"}]},
                {"address":"two.example","port":8443,"users":[{"id":"00000000-0000-4000-8000-000000000003"}]}
            ]}}]
        }"#,
        &source,
        0,
    );

    assert_eq!(configs.len(), 3);
    assert_eq!(configs[0].host, "one.example");
    assert_eq!(
        configs[1].uuid.as_deref(),
        Some("00000000-0000-4000-8000-000000000002")
    );
    assert_eq!(configs[2].host, "two.example");
    assert_ne!(configs[0].id, configs[1].id);
    assert_ne!(configs[1].id, configs[2].id);
}

#[test]
fn json_resolver_marks_unrepresentable_transport_fields_as_limited() {
    let source = url::Url::parse("https://example.test/profile").unwrap();
    let config = parse_configurations(
        include_str!("fixtures/ready_xray_hysteria2_limits.json"),
        &source,
        0,
    )
    .pop()
    .unwrap();

    assert!(matches!(config.share_uri, ShareUriResult::Limited { .. }));
    assert!(config.share_uri.uri().is_some_and(|uri| uri
        .contains("hop.example.invalid:443,5000-6000")
        && uri.contains("alpn=h3")));
    assert!(config
        .share_uri
        .limitations()
        .iter()
        .any(|limitation| limitation.to_string().contains("congestionControl")));
}

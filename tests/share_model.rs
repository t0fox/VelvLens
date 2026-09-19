use sublens::{
    model::{OriginalRepresentation, ShareUriResult},
    protocols::parse_uri,
};

#[test]
fn ordinary_uri_keeps_exact_original_and_available_share_uri() {
    let source = "vless://12345678-1234-1234-1234-123456789abc@example.test:443?security=reality&x-custom=one#Demo";
    let config = parse_uri(source, None, 0).unwrap();

    assert_eq!(config.original_text(), source);
    assert!(!config.original_is_json());
    assert!(matches!(
        &config.original,
        OriginalRepresentation::ShareUri(value) if value == source
    ));
    assert!(matches!(
        &config.share_uri,
        ShareUriResult::Available { uri } if uri == source
    ));
}

#[test]
fn unavailable_share_uri_has_no_copyable_uri() {
    let result = ShareUriResult::Unavailable {
        reasons: vec![sublens::model::ConversionLimitation::MissingRequired {
            field: "uuid".to_owned(),
        }],
    };

    assert_eq!(result.uri(), None);
    assert!(result.is_unavailable());
}

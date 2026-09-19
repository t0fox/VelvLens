use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::header::{HeaderMap, HeaderValue};

use sublens::resolver::metadata::{parse_body_directives, SubscriptionMetadata};
use sublens::resolver::{Resolver, ResolverConfig, SourceClassification};
use tokio_util::sync::CancellationToken;

#[test]
fn captures_happ_body_directives_and_decodes_profile_title() {
    let title = STANDARD.encode("Primary profile");
    let metadata = parse_body_directives(&format!(
        "#profile-title: {title}\n#profile-update-interval: 3600\n#support-url: https://support.example\n#change-user-agent: Mozilla/5.0\n#provider-note: keep\n"
    ));
    assert_eq!(metadata.profile_title.as_deref(), Some("Primary profile"));
    assert_eq!(metadata.profile_update_interval, Some(3600));
    assert_eq!(
        metadata.support_url.as_deref(),
        Some("https://support.example")
    );
    assert_eq!(metadata.directives["change-user-agent"], "Mozilla/5.0");
    assert_eq!(metadata.directives["provider-note"], "keep");
}

#[test]
fn captures_documented_happ_headers_without_logging_raw_payloads() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "profile-title",
        HeaderValue::from_static("UHJpbWFyeSBwcm9maWxl"),
    );
    headers.insert("profile-update-interval", HeaderValue::from_static("3600"));
    headers.insert(
        "subscription-userinfo",
        HeaderValue::from_static("upload=1; download=2; total=3; expire=4"),
    );
    headers.insert(
        "support-url",
        HeaderValue::from_static("https://support.example"),
    );
    headers.insert("change-user-agent", HeaderValue::from_static("Mozilla/5.0"));
    headers.insert("dont-use-filter", HeaderValue::from_static("true"));
    let metadata = SubscriptionMetadata::from_headers(&headers);
    assert_eq!(metadata.profile_title.as_deref(), Some("Primary profile"));
    assert_eq!(metadata.profile_update_interval, Some(3600));
    assert_eq!(
        metadata.subscription_userinfo.as_deref(),
        Some("upload=1; download=2; total=3; expire=4")
    );
    assert_eq!(
        metadata.support_url.as_deref(),
        Some("https://support.example")
    );
    assert_eq!(metadata.directives["change-user-agent"], "Mozilla/5.0");
    assert_eq!(metadata.directives["dont-use-filter"], "true");
}

#[tokio::test]
async fn encrypted_happ_links_are_classified_without_decryption() {
    let report = Resolver::new(reqwest::Client::new(), ResolverConfig::default())
        .analyze("happ://crypt5/public-example", CancellationToken::new())
        .await
        .unwrap();
    assert!(report.configs.is_empty());
    assert_eq!(
        report.source_classification,
        Some(SourceClassification::EncryptedHappSubscription)
    );
    assert!(report
        .stages
        .iter()
        .any(|stage| stage.preview.contains("Encrypted HAPP subscription")));
    assert!(report
        .stages
        .iter()
        .filter_map(|stage| stage.error.as_deref())
        .any(|error| error.contains("not attempted")));
}
